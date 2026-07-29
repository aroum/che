use std::{collections::HashSet, io, process::Stdio, sync::Arc};

use tokio::process::Command;
use yazi_fs::{cha::{Cha, ChaKind, ChaMode}, provider::{DirReader, FileHolder}};
use yazi_shared::{path::PathBufDyn, strand::StrandCow, url::{AsUrl, UrlBuf, UrlLike}};

pub struct ReadDir {
	dir:     Arc<UrlBuf>,
	entries: Vec<ArchiveEntry>,
	idx:     usize,
}

pub struct ArchiveEntry {
	name:   String,
	is_dir: bool,
	size:   u64,
}

impl ReadDir {
	pub async fn new(url: &UrlBuf) -> io::Result<Self> {
		let (base_path, target_dir) = match url.as_url() {
			yazi_shared::url::Url::Archive { loc, .. } => {
				let (base, rest, urn) = loc.triple();
				let target = format!("{}{}", rest.to_string_lossy(), urn.to_string_lossy()).trim_matches('/').to_string();
				let clean_base = std::path::PathBuf::from(base.to_string_lossy().trim_end_matches('/'));
				(clean_base, target)
			}
			_ => (url.loc().as_os().map_or_else(|_| std::path::PathBuf::new(), std::path::PathBuf::from), String::new()),
		};

		tracing::info!("Archive ReadDir::new: url={url:?}, base_path={base_path:?}, target_dir={target_dir:?}");

		let mut cmd = Command::new("7zz");
		cmd.args(["l", "-ba", "-slt", "-sccUTF-8", "-xr!__MACOSX"]);
		cmd.arg(&base_path);
		cmd.stdout(Stdio::piped()).stderr(Stdio::null());

		let output = match cmd.output().await {
			Ok(o) => o,
			Err(_) => {
				let mut cmd2 = Command::new("7z");
				cmd2.args(["l", "-ba", "-slt", "-sccUTF-8", "-xr!__MACOSX"]);
				cmd2.arg(&base_path);
				cmd2.stdout(Stdio::piped()).stderr(Stdio::null());
				cmd2.output().await?
			}
		};

		let stdout = String::from_utf8_lossy(&output.stdout);
		let mut raw_entries = Vec::new();

		let mut cur_path = String::new();
		let mut cur_size = 0u64;
		let mut cur_is_dir = false;

		for line in stdout.lines() {
			let line = line.trim();
			if line.is_empty() {
				if !cur_path.is_empty() {
					raw_entries.push((cur_path.clone(), cur_is_dir, cur_size));
					cur_path.clear();
					cur_size = 0;
					cur_is_dir = false;
				}
				continue;
			}

			if let Some(val) = line.strip_prefix("Path = ") {
				cur_path = val.replace('\\', "/").trim_matches('/').to_string();
			} else if let Some(val) = line.strip_prefix("Size = ") {
				cur_size = val.parse().unwrap_or(0);
			} else if let Some(val) = line.strip_prefix("Folder = ") {
				cur_is_dir = val == "+";
			} else if let Some(val) = line.strip_prefix("Attributes = ") {
				if val.starts_with('D') {
					cur_is_dir = true;
				}
			}
		}

		let target_dir_norm = target_dir.replace('\\', "/").trim_matches('/').to_string();
		let prefix = if target_dir_norm.is_empty() { String::new() } else { format!("{target_dir_norm}/") };
		let mut seen_subdirs = HashSet::new();
		let mut entries = Vec::new();

		for (path, is_dir, size) in raw_entries {
			if !prefix.is_empty() && !path.starts_with(&prefix) {
				continue;
			}

			let rel_path = &path[prefix.len()..];
			if rel_path.is_empty() {
				continue;
			}

			if let Some((dir_name, _)) = rel_path.split_once('/') {
				if seen_subdirs.insert(dir_name.to_string()) {
					entries.push(ArchiveEntry {
						name: dir_name.to_string(),
						is_dir: true,
						size: 0,
					});
				}
			} else if is_dir {
				if seen_subdirs.insert(rel_path.to_string()) {
					entries.push(ArchiveEntry {
						name: rel_path.to_string(),
						is_dir: true,
						size: 0,
					});
				}
			} else {
				entries.push(ArchiveEntry {
					name: rel_path.to_string(),
					is_dir,
					size,
				});
			}
		}

		tracing::info!("Archive ReadDir::new: found {} entries for target_dir_norm '{target_dir_norm}' (orig '{target_dir}')", entries.len());

		Ok(Self {
			dir: Arc::new(url.clone()),
			entries,
			idx: 0,
		})
	}
}

impl DirReader for ReadDir {
	type Entry = DirEntry;

	async fn next(&mut self) -> io::Result<Option<Self::Entry>> {
		if self.idx >= self.entries.len() {
			return Ok(None);
		}

		let item = &self.entries[self.idx];
		self.idx += 1;

		Ok(Some(DirEntry {
			dir: self.dir.clone(),
			name: item.name.clone(),
			is_dir: item.is_dir,
			size: item.size,
		}))
	}
}

pub struct DirEntry {
	dir:    Arc<UrlBuf>,
	name:   String,
	is_dir: bool,
	size:   u64,
}

impl FileHolder for DirEntry {
	async fn file_type(&self) -> io::Result<yazi_fs::cha::ChaType> {
		Ok(if self.is_dir {
			yazi_fs::cha::ChaType::Dir
		} else {
			yazi_fs::cha::ChaType::File
		})
	}

	async fn metadata(&self) -> io::Result<Cha> {
		let mode = if self.is_dir { ChaMode::T_DIR } else { ChaMode::T_FILE };
		Ok(Cha {
			kind: ChaKind::empty(),
			mode,
			len: self.size,
			..Default::default()
		})
	}

	fn name(&self) -> StrandCow<'_> {
		StrandCow::from(self.name.as_str())
	}

	fn path(&self) -> PathBufDyn {
		PathBufDyn::from(std::path::PathBuf::from(&self.name))
	}

	fn url(&self) -> UrlBuf {
		self.dir.try_join(&self.name).unwrap_or_else(|_| self.dir.as_ref().clone())
	}
}

pub async fn open(url: &UrlBuf) -> io::Result<super::RwFile> {
	let (base_path, target_file) = match url.as_url() {
		yazi_shared::url::Url::Archive { loc, .. } => {
			let (base, rest, urn) = loc.triple();
			let target = format!("{}{}", rest.to_string_lossy(), urn.to_string_lossy()).trim_matches('/').to_string();
			let clean_base = std::path::PathBuf::from(base.to_string_lossy().trim_end_matches('/'));
			(clean_base, target)
		}
		_ => return Err(io::Error::new(io::ErrorKind::InvalidInput, "Not an archive URL")),
	};

	let mut cmd = Command::new("7zz");
	cmd.args(["x", "-so", "-sccUTF-8", "-xr!__MACOSX"]);
	cmd.arg(&base_path);
	cmd.arg(&target_file);
	cmd.stdout(Stdio::piped()).stderr(Stdio::null());

	let output = match cmd.output().await {
		Ok(o) => o,
		Err(_) => {
			let mut cmd2 = Command::new("7z");
			cmd2.args(["x", "-so", "-sccUTF-8", "-xr!__MACOSX"]);
			cmd2.arg(&base_path);
			cmd2.arg(&target_file);
			cmd2.stdout(Stdio::piped()).stderr(Stdio::null());
			cmd2.output().await?
		}
	};

	Ok(super::RwFile::Archive(std::io::Cursor::new(output.stdout)))
}

pub async fn copy(from: &UrlBuf, to: &UrlBuf) -> io::Result<u64> {
	let (base_path, target_file) = match to.as_url() {
		yazi_shared::url::Url::Archive { loc, .. } => {
			let (base, rest, urn) = loc.triple();
			let target = format!("{}{}", rest.to_string_lossy(), urn.to_string_lossy()).trim_matches('/').to_string();
			let clean_base = std::path::PathBuf::from(base.to_string_lossy().trim_end_matches('/'));
			(clean_base, target)
		}
		_ => return Err(io::Error::new(io::ErrorKind::InvalidInput, "Target is not an archive URL")),
	};

	let mut src = super::provider::open(from).await?;
	let mut content = Vec::new();
	tokio::io::AsyncReadExt::read_to_end(&mut src, &mut content).await?;
	let len = content.len() as u64;

	let mut cmd = Command::new("7zz");
	cmd.arg("a");
	cmd.arg(format!("-si{target_file}"));
	cmd.arg("-sccUTF-8");
	cmd.arg(&base_path);
	cmd.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null());

	let mut child = match cmd.spawn() {
		Ok(c) => c,
		Err(_) => {
			let mut cmd2 = Command::new("7z");
			cmd2.arg("a");
			cmd2.arg(format!("-si{target_file}"));
			cmd2.arg("-sccUTF-8");
			cmd2.arg(&base_path);
			cmd2.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null());
			cmd2.spawn()?
		}
	};

	if let Some(mut stdin) = child.stdin.take() {
		tokio::io::AsyncWriteExt::write_all(&mut stdin, &content).await?;
	}

	let output = child.wait_with_output().await?;
	if !output.status.success() {
		return Err(io::Error::new(io::ErrorKind::Other, format!("Failed to add {target_file} to archive")));
	}

	Ok(len)
}

pub async fn remove_file(url: &UrlBuf) -> io::Result<()> {
	let (base_path, target_file) = match url.as_url() {
		yazi_shared::url::Url::Archive { loc, .. } => {
			let (base, rest, urn) = loc.triple();
			let target = format!("{}{}", rest.to_string_lossy(), urn.to_string_lossy()).trim_matches('/').to_string();
			let clean_base = std::path::PathBuf::from(base.to_string_lossy().trim_end_matches('/'));
			(clean_base, target)
		}
		_ => return Err(io::Error::new(io::ErrorKind::InvalidInput, "Not an archive URL")),
	};

	let mut cmd = Command::new("7zz");
	cmd.args(["d", "-sccUTF-8"]);
	cmd.arg(&base_path);
	cmd.arg(&target_file);
	cmd.stdout(Stdio::piped()).stderr(Stdio::null());

	let output = match cmd.output().await {
		Ok(o) => o,
		Err(_) => {
			let mut cmd2 = Command::new("7z");
			cmd2.args(["d", "-sccUTF-8"]);
			cmd2.arg(&base_path);
			cmd2.arg(&target_file);
			cmd2.stdout(Stdio::piped()).stderr(Stdio::null());
			cmd2.output().await?
		}
	};

	if !output.status.success() {
		return Err(io::Error::new(io::ErrorKind::Other, format!("Failed to remove {target_file} from archive")));
	}

	Ok(())
}

pub async fn remove_dir(url: &UrlBuf) -> io::Result<()> {
	let (base_path, target_dir) = match url.as_url() {
		yazi_shared::url::Url::Archive { loc, .. } => {
			let (base, rest, urn) = loc.triple();
			let target = format!("{}{}", rest.to_string_lossy(), urn.to_string_lossy()).trim_matches('/').to_string();
			let clean_base = std::path::PathBuf::from(base.to_string_lossy().trim_end_matches('/'));
			(clean_base, target)
		}
		_ => return Err(io::Error::new(io::ErrorKind::InvalidInput, "Not an archive URL")),
	};

	let pattern = format!("{target_dir}/*");
	let mut cmd = Command::new("7zz");
	cmd.args(["d", "-r", "-sccUTF-8"]);
	cmd.arg(&base_path);
	cmd.arg(&pattern);
	cmd.arg(&target_dir);
	cmd.stdout(Stdio::piped()).stderr(Stdio::null());

	let output = match cmd.output().await {
		Ok(o) => o,
		Err(_) => {
			let mut cmd2 = Command::new("7z");
			cmd2.args(["d", "-r", "-sccUTF-8"]);
			cmd2.arg(&base_path);
			cmd2.arg(&pattern);
			cmd2.arg(&target_dir);
			cmd2.stdout(Stdio::piped()).stderr(Stdio::null());
			cmd2.output().await?
		}
	};

	if !output.status.success() {
		return Err(io::Error::new(io::ErrorKind::Other, format!("Failed to remove directory {target_dir} from archive")));
	}

	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::path::PathBuf;

	#[tokio::test]
	async fn test_archive_entry_metadata() {
		let entry = DirEntry {
			dir: Arc::new(UrlBuf::default()),
			name: "test.txt".to_string(),
			is_dir: false,
			size: 1234,
		};

		let meta = entry.metadata().await.unwrap();
		assert_eq!(meta.len, 1234);
		assert_eq!(meta.is_dir(), false);
	}

	#[tokio::test]
	async fn test_read_real_test_zip() {
		let test_zip = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/test.zip");
		if !test_zip.exists() {
			return;
		}

		if std::panic::catch_unwind(|| yazi_shared::init()).is_err() {
			// Already initialized
		}

		// Root of archive
		let url = UrlBuf::from(test_zip.clone()).into_archive("1").unwrap();
		let mut reader = ReadDir::new(&url).await.unwrap();

		let mut root_entries = Vec::new();
		while let Ok(Some(entry)) = reader.next().await {
			root_entries.push(entry.name.to_string());
		}

		assert_eq!(root_entries, vec!["test_folder"]);

		// Subfolder inside archive
		let sub_url = url.try_join("test_folder").unwrap();
		let mut sub_reader = ReadDir::new(&sub_url).await.unwrap();

		let mut sub_entries = Vec::new();
		while let Ok(Some(entry)) = sub_reader.next().await {
			sub_entries.push(entry.name.to_string());
		}

		assert!(sub_entries.contains(&"README.md".to_string()));
		assert!(sub_entries.contains(&"AGENTS.md".to_string()));
		assert!(sub_entries.contains(&"subfolder".to_string()));
	}

	#[tokio::test]
	async fn test_archive_operations_protection() {
		let test_zip = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/test.zip");
		if !test_zip.exists() {
			return;
		}

		if std::panic::catch_unwind(|| yazi_shared::init()).is_err() {
			// Already initialized
		}

		let url = UrlBuf::from(test_zip.clone()).into_archive("1").unwrap();
		let sub_file = url.try_join("test_folder/README.md").unwrap();

		// 1. unique_file: should succeed and return URL
		let uniq = crate::fns::unique_file(sub_file.clone(), false).await;
		assert!(uniq.is_ok());
		assert_eq!(uniq.unwrap(), sub_file);

		// 2. create_dir / create_dir_all: should succeed gracefully
		assert!(crate::provider::create_dir(&sub_file).await.is_ok());
		assert!(crate::provider::create_dir_all(&sub_file).await.is_ok());

		// 3. create_new: should return PermissionDenied
		assert_eq!(
			crate::provider::create_new(&sub_file).await.err().map(|e| e.kind()),
			Some(std::io::ErrorKind::PermissionDenied)
		);

		// 4. rename / trash: should return PermissionDenied
		assert_eq!(
			crate::provider::rename(&sub_file, &url).await.err().map(|e| e.kind()),
			Some(std::io::ErrorKind::PermissionDenied)
		);
		assert_eq!(
			crate::provider::trash(&sub_file).await.err().map(|e| e.kind()),
			Some(std::io::ErrorKind::PermissionDenied)
		);
	}

	#[tokio::test]
	async fn test_archive_open_and_copy_out() {
		use tokio::io::AsyncReadExt;
		let test_zip = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/test.zip");
		if !test_zip.exists() {
			return;
		}

		if std::panic::catch_unwind(|| yazi_shared::init()).is_err() {
			// Already initialized
		}

		let url = UrlBuf::from(test_zip.clone()).into_archive("1").unwrap();
		let sub_file = url.try_join("test_folder/subfolder/sample.txt").unwrap();

		// 1. open archive file
		let mut rw_file = crate::provider::open(&sub_file).await.unwrap();
		let mut buf = Vec::new();
		rw_file.read_to_end(&mut buf).await.unwrap();
		let content = String::from_utf8_lossy(&buf);
		assert!(content.contains("Sample text file"));

		// 2. copy file from archive to local file
		let tmp_dest = std::env::temp_dir().join("test_copy_out_sample.txt");
		let copied_bytes = crate::provider::copy(&sub_file, &tmp_dest, yazi_fs::provider::Attrs::default()).await.unwrap();
		assert!(copied_bytes > 0);

		let read_back = tokio::fs::read_to_string(&tmp_dest).await.unwrap();
		assert_eq!(read_back, content);

		// 3. copy file into temporary zip archive
		let tmp_zip = std::env::temp_dir().join("test_copy_into.zip");
		let _ = tokio::fs::copy(&test_zip, &tmp_zip).await;

		let archive_target_url = UrlBuf::from(tmp_zip.clone()).into_archive("1").unwrap();
		let target_file_url = archive_target_url.try_join("test_folder/added_test.txt").unwrap();

		let added_bytes = crate::provider::copy(&tmp_dest, &target_file_url, yazi_fs::provider::Attrs::default()).await.unwrap();
		assert!(added_bytes > 0);

		// Verify added file exists inside archive and can be opened
		let mut added_file = crate::provider::open(&target_file_url).await.unwrap();
		let mut added_buf = Vec::new();
		added_file.read_to_end(&mut added_buf).await.unwrap();
		assert_eq!(String::from_utf8_lossy(&added_buf), content);

		// 4. remove file from archive
		let remove_res = crate::provider::remove_file(&target_file_url).await;
		assert!(remove_res.is_ok(), "remove_file failed: {:?}", remove_res.err());

		let _ = tokio::fs::remove_file(tmp_dest).await;
		let _ = tokio::fs::remove_file(tmp_zip).await;
	}
}
