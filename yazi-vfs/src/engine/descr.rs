use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

fn parse_line(line: &str) -> Option<(String, String)> {
	let line = line.trim();
	if line.is_empty() {
		return None;
	}

	if let Some(rest) = line.strip_prefix('"') {
		let (filename, comment) = rest.split_once('"')?;
		Some((filename.to_string(), comment.trim().to_string()))
	} else if let Some((filename, comment)) = line.split_once(' ') {
		Some((filename.to_string(), comment.trim().to_string()))
	} else {
		Some((line.to_string(), String::new()))
	}
}

pub fn read_description(file_path: &Path) -> Option<String> {
    let parent = file_path.parent()?;
    let descr_file = parent.join("descript.ion");
    if !descr_file.exists() {
        return None;
    }

    let file = fs::File::open(&descr_file).ok()?;
    let reader = BufReader::new(file);
    let target_name = file_path.file_name()?.to_string_lossy();

    for line in reader.lines().map_while(Result::ok) {
        if let Some((filename, comment)) = parse_line(&line) {
            if filename.eq_ignore_ascii_case(&target_name) {
                return Some(comment);
            }
        }
    }

    None
}

pub fn write_description(file_path: &Path, descr: &str) {
    let Some(parent) = file_path.parent() else { return };
    let descr_file = parent.join("descript.ion");
    let Some(target_name) = file_path.file_name().map(|s| s.to_string_lossy().to_string()) else { return };

    let mut lines = Vec::new();
    let mut found = false;

    if descr_file.exists() {
        if let Ok(file) = fs::File::open(&descr_file) {
            let reader = BufReader::new(file);
            for line in reader.lines().map_while(Result::ok) {
                if let Some((filename, comment)) = parse_line(&line) {
                    if filename.eq_ignore_ascii_case(&target_name) {
                        found = true;
                        if !descr.is_empty() {
                            lines.push((filename, descr.to_string()));
                        }
                    } else {
                        lines.push((filename, comment));
                    }
                }
            }
        }
    }

    if !found && !descr.is_empty() {
        lines.push((target_name, descr.to_string()));
    }

    if lines.is_empty() {
        if descr_file.exists() {
            fs::remove_file(&descr_file).ok();
        }
    } else if let Ok(mut file) = fs::File::create(&descr_file) {
        for (filename, comment) in lines {
            let line = if filename.contains(' ') {
                format!("\"{}\" {}\n", filename, comment)
            } else {
                format!("{} {}\n", filename, comment)
            };
            file.write_all(line.as_bytes()).ok();
        }
    }
}

pub fn copy_description(from: &Path, to: &Path) {
    if let Some(descr) = read_description(from) {
        write_description(to, &descr);
    }
}

pub fn move_description(from: &Path, to: &Path) {
    if let Some(descr) = read_description(from) {
        write_description(to, &descr);
        write_description(from, "");
    }
}
