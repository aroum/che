use std::{
	fs,
	io::stdout,
	path::{Path, PathBuf},
};

use anyhow::Result;
use crossterm::{
	event::{
		self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers,
		MouseButton, MouseEventKind,
	},
	execute,
	terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
	Terminal,
	backend::CrosstermBackend,
	layout::{Alignment, Constraint, Direction, Layout, Rect},
	style::{Color, Modifier, Style},
	text::{Line, Span},
	widgets::{Block, Borders, Clear, Paragraph},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum ArchiveFormat {
	SevenZip,
	Zip,
	TarGz,
	TarXz,
	TarBz2,
	TarZst,
	Tar,
}

impl ArchiveFormat {
	pub fn extension(&self) -> &'static str {
		match self {
			Self::SevenZip => "7z",
			Self::Zip => "zip",
			Self::TarGz => "tar.gz",
			Self::TarXz => "tar.xz",
			Self::TarBz2 => "tar.bz2",
			Self::TarZst => "tar.zst",
			Self::Tar => "tar",
		}
	}

	pub fn label(&self) -> &'static str {
		match self {
			Self::SevenZip => "7z",
			Self::Zip => "zip",
			Self::TarGz => "tar.gz",
			Self::TarXz => "tar.xz",
			Self::TarBz2 => "tar.bz2",
			Self::TarZst => "tar.zst",
			Self::Tar => "tar",
		}
	}

	pub fn all() -> &'static [Self] {
		&[Self::SevenZip, Self::Zip, Self::TarGz, Self::TarXz, Self::TarBz2, Self::TarZst]
	}

	pub fn from_ext(ext: &str) -> Self {
		match ext.to_lowercase().as_str() {
			"zip" => Self::Zip,
			"tar.gz" | "tgz" => Self::TarGz,
			"tar.xz" | "txz" => Self::TarXz,
			"tar.bz2" | "tbz2" => Self::TarBz2,
			"tar.zst" | "tzst" => Self::TarZst,
			"tar" => Self::Tar,
			_ => Self::SevenZip,
		}
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum CompressionLevel {
	Store,
	Fastest,
	Fast,
	Normal,
	Maximum,
	Ultra,
}

impl CompressionLevel {
	pub fn value(&self) -> u8 {
		match self {
			Self::Store => 0,
			Self::Fastest => 1,
			Self::Fast => 3,
			Self::Normal => 5,
			Self::Maximum => 7,
			Self::Ultra => 9,
		}
	}

	pub fn label(&self) -> &'static str {
		match self {
			Self::Store => "Store",
			Self::Fastest => "Fastest",
			Self::Fast => "Fast",
			Self::Normal => "Normal",
			Self::Maximum => "Maximum",
			Self::Ultra => "Ultra",
		}
	}

	pub fn all() -> &'static [Self] {
		&[Self::Store, Self::Fastest, Self::Fast, Self::Normal, Self::Maximum, Self::Ultra]
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum CompressionMethod {
	Auto,
	LZMA2,
	LZMA,
	Deflate,
	BZip2,
	ZSTD,
	Copy,
}

impl CompressionMethod {
	pub fn label(&self) -> &'static str {
		match self {
			Self::Auto => "Auto",
			Self::LZMA2 => "LZMA2",
			Self::LZMA => "LZMA",
			Self::Deflate => "Deflate",
			Self::BZip2 => "BZip2",
			Self::ZSTD => "ZSTD",
			Self::Copy => "Copy",
		}
	}

	pub fn all() -> &'static [Self] {
		&[Self::Auto, Self::LZMA2, Self::LZMA, Self::Deflate, Self::BZip2, Self::ZSTD]
	}
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct PersistentArchiveState {
	pub format: Option<ArchiveFormat>,
	pub level_idx: Option<usize>,
	pub method_idx: Option<usize>,
	pub solid: Option<bool>,
	pub delete_source: Option<bool>,
	pub overwrite: Option<bool>,
}

fn state_path() -> PathBuf {
	if let Some(home) = std::env::var_os("HOME") {
		let dir = PathBuf::from(home).join(".config/che");
		let _ = fs::create_dir_all(&dir);
		dir.join("archive_state.json")
	} else {
		PathBuf::from("/tmp/che_archive_state.json")
	}
}

fn load_state() -> PersistentArchiveState {
	let p = state_path();
	if let Ok(f) = fs::File::open(p) {
		serde_json::from_reader(f).unwrap_or_default()
	} else {
		PersistentArchiveState::default()
	}
}

fn save_state(state: &PersistentArchiveState) {
	let p = state_path();
	if let Ok(f) = fs::File::create(p) {
		let _ = serde_json::to_writer(f, state);
	}
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Focus {
	ArchivePath,
	Format,
	Level,
	Method,
	Solid,
	Encrypt,
	EncryptHeader,
	ShowPassword,
	Password,
	RepeatPassword,
	DeleteSource,
	DestPath,
	DestCurrent,
	DestSubdir,
	DestOpposite,
	Overwrite,
	DeleteArchive,
	OkBtn,
	CancelBtn,
}

#[derive(Clone, Debug)]
pub struct TextInput {
	pub value: String,
	pub cursor: usize,
}

impl TextInput {
	pub fn new(val: &str) -> Self {
		let len = val.chars().count();
		Self { value: val.to_string(), cursor: len }
	}

	pub fn insert(&mut self, c: char) {
		let mut chars: Vec<char> = self.value.chars().collect();
		if self.cursor > chars.len() {
			self.cursor = chars.len();
		}
		chars.insert(self.cursor, c);
		self.value = chars.into_iter().collect();
		self.cursor += 1;
	}

	pub fn backspace(&mut self) {
		if self.cursor > 0 {
			let mut chars: Vec<char> = self.value.chars().collect();
			if self.cursor <= chars.len() {
				chars.remove(self.cursor - 1);
				self.value = chars.into_iter().collect();
				self.cursor -= 1;
			}
		}
	}

	pub fn delete(&mut self) {
		let mut chars: Vec<char> = self.value.chars().collect();
		if self.cursor < chars.len() {
			chars.remove(self.cursor);
			self.value = chars.into_iter().collect();
		}
	}

	pub fn move_left(&mut self) {
		if self.cursor > 0 {
			self.cursor -= 1;
		}
	}

	pub fn move_right(&mut self) {
		let len = self.value.chars().count();
		if self.cursor < len {
			self.cursor += 1;
		}
	}

	pub fn move_home(&mut self) {
		self.cursor = 0;
	}

	pub fn move_end(&mut self) {
		self.cursor = self.value.chars().count();
	}

	pub fn word_left(&mut self) {
		let chars: Vec<char> = self.value.chars().collect();
		if self.cursor == 0 {
			return;
		}
		let mut idx = self.cursor;
		while idx > 0 && chars[idx - 1].is_whitespace() {
			idx -= 1;
		}
		while idx > 0 && !chars[idx - 1].is_whitespace() {
			idx -= 1;
		}
		self.cursor = idx;
	}

	pub fn word_right(&mut self) {
		let chars: Vec<char> = self.value.chars().collect();
		let len = chars.len();
		if self.cursor >= len {
			return;
		}
		let mut idx = self.cursor;
		while idx < len && !chars[idx].is_whitespace() {
			idx += 1;
		}
		while idx < len && chars[idx].is_whitespace() {
			idx += 1;
		}
		self.cursor = idx;
	}

	pub fn delete_word_left(&mut self) {
		let old_cursor = self.cursor;
		self.word_left();
		let new_cursor = self.cursor;
		let mut chars: Vec<char> = self.value.chars().collect();
		for _ in new_cursor..old_cursor {
			if new_cursor < chars.len() {
				chars.remove(new_cursor);
			}
		}
		self.value = chars.into_iter().collect();
	}

	pub fn delete_word_right(&mut self) {
		let old_cursor = self.cursor;
		self.word_right();
		let target_cursor = self.cursor;
		self.cursor = old_cursor;
		let mut chars: Vec<char> = self.value.chars().collect();
		for _ in old_cursor..target_cursor {
			if old_cursor < chars.len() {
				chars.remove(old_cursor);
			}
		}
		self.value = chars.into_iter().collect();
	}

	pub fn delete_to_start(&mut self) {
		let chars: Vec<char> = self.value.chars().collect();
		if self.cursor <= chars.len() {
			self.value = chars[self.cursor..].iter().collect();
			self.cursor = 0;
		}
	}

	pub fn delete_to_end(&mut self) {
		let chars: Vec<char> = self.value.chars().collect();
		if self.cursor <= chars.len() {
			self.value = chars[..self.cursor].iter().collect();
		}
	}

	pub fn render_spans<'a>(
		&'a self,
		is_focused: bool,
		normal_style: Style,
		active_style: Style,
	) -> Vec<Span<'a>> {
		if !is_focused {
			return vec![Span::styled(&self.value, normal_style)];
		}

		let chars: Vec<char> = self.value.chars().collect();
		let cursor_style =
			Style::default().bg(Color::White).fg(Color::Black).add_modifier(Modifier::BOLD);

		if chars.is_empty() {
			return vec![Span::styled(" ", cursor_style)];
		}

		let mut spans = Vec::new();
		let cursor_pos = self.cursor.min(chars.len());

		if cursor_pos > 0 {
			let before: String = chars[..cursor_pos].iter().collect();
			spans.push(Span::styled(before, active_style));
		}

		if cursor_pos < chars.len() {
			let at_cursor: String = chars[cursor_pos..cursor_pos + 1].iter().collect();
			spans.push(Span::styled(at_cursor, cursor_style));
			if cursor_pos + 1 < chars.len() {
				let after: String = chars[cursor_pos + 1..].iter().collect();
				spans.push(Span::styled(after, active_style));
			}
		} else {
			spans.push(Span::styled(" ", cursor_style));
		}

		spans
	}

	pub fn render_password_spans<'a>(
		&'a self,
		is_focused: bool,
		show_password: bool,
		normal_style: Style,
		active_style: Style,
	) -> Vec<Span<'a>> {
		if show_password {
			return self.render_spans(is_focused, normal_style, active_style);
		}

		let count = self.value.chars().count();
		let cursor_style =
			Style::default().bg(Color::White).fg(Color::Black).add_modifier(Modifier::BOLD);

		if !is_focused {
			if count == 0 {
				return vec![Span::styled("<empty>", normal_style)];
			}
			return vec![Span::styled("*".repeat(count), normal_style)];
		}

		if count == 0 {
			return vec![Span::styled(" ", cursor_style)];
		}

		let mut spans = Vec::new();
		let cursor_pos = self.cursor.min(count);

		if cursor_pos > 0 {
			spans.push(Span::styled("*".repeat(cursor_pos), active_style));
		}

		if cursor_pos < count {
			spans.push(Span::styled("*", cursor_style));
			if cursor_pos + 1 < count {
				spans.push(Span::styled("*".repeat(count - cursor_pos - 1), active_style));
			}
		} else {
			spans.push(Span::styled(" ", cursor_style));
		}

		spans
	}
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct ArchiveResult {
	pub op: String,
	pub format: String,
	pub archive_path: String,
	pub target_dir: String,
	pub files: Vec<String>,
	pub level: u8,
	pub method: String,
	pub solid: bool,
	pub password: Option<String>,
	pub encrypt_header: bool,
	pub delete_source: bool,
	pub overwrite: bool,
}

pub fn run(
	files: Vec<String>,
	mode: String,
	output_dir: Option<String>,
	opposite_dir: Option<String>,
) -> Result<()> {
	if files.is_empty() {
		return Ok(());
	}

	enable_raw_mode()?;
	let mut out = stdout();
	execute!(out, EnterAlternateScreen, EnableMouseCapture)?;
	let backend = CrosstermBackend::new(out);
	let mut terminal = Terminal::new(backend)?;

	let app_result = run_app(&mut terminal, files, mode, output_dir, opposite_dir);

	disable_raw_mode()?;
	execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
	terminal.show_cursor()?;

	app_result
}

pub fn is_archive_file(path_str: &str) -> bool {
	let lower = path_str.to_lowercase();
	lower.ends_with(".zip")
		|| lower.ends_with(".7z")
		|| lower.ends_with(".rar")
		|| lower.ends_with(".tar")
		|| lower.ends_with(".tar.gz")
		|| lower.ends_with(".tgz")
		|| lower.ends_with(".tar.xz")
		|| lower.ends_with(".txz")
		|| lower.ends_with(".tar.bz2")
		|| lower.ends_with(".tbz2")
		|| lower.ends_with(".tar.zst")
		|| lower.ends_with(".tzst")
}

fn run_app(
	terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
	files: Vec<String>,
	mode_arg: String,
	output_dir: Option<String>,
	opposite_dir: Option<String>,
) -> Result<()> {
	let is_extract = if mode_arg.starts_with("extract") {
		true
	} else if mode_arg.starts_with("pack") {
		false
	} else {
		files.len() == 1 && is_archive_file(&files[0])
	};

	if is_extract {
		run_extract_app(terminal, files, mode_arg, output_dir, opposite_dir)
	} else {
		run_pack_app(terminal, files, mode_arg, output_dir, opposite_dir)
	}
}

fn run_pack_app(
	terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
	files: Vec<String>,
	mode_arg: String,
	output_dir: Option<String>,
	opposite_dir: Option<String>,
) -> Result<()> {
	let persistent = load_state();

	let first_path = Path::new(&files[0]);
	let base_name = if files.len() == 1 {
		first_path.file_stem().and_then(|s| s.to_str()).unwrap_or("archive")
	} else {
		first_path.parent().and_then(|p| p.file_name()).and_then(|s| s.to_str()).unwrap_or("archive")
	};

	let chosen_dir = if mode_arg == "pack-opposite" && opposite_dir.is_some() {
		opposite_dir.as_deref().map(Path::new).unwrap_or(Path::new("."))
	} else {
		output_dir
			.as_deref()
			.map(Path::new)
			.or_else(|| first_path.parent())
			.unwrap_or_else(|| Path::new("."))
	};

	let mut format = persistent.format.unwrap_or(ArchiveFormat::SevenZip);
	let archive_name = format!("{}.{}", base_name, format.extension());
	let mut path_input = TextInput::new(&chosen_dir.join(&archive_name).to_string_lossy());

	let mut level_idx = persistent.level_idx.unwrap_or(3);
	let mut method_idx = persistent.method_idx.unwrap_or(0);
	let mut solid = persistent.solid.unwrap_or(true);
	let mut encrypt = false;
	let mut encrypt_header = false;
	let mut show_password = false;
	let mut password_input = TextInput::new("");
	let mut repeat_input = TextInput::new("");
	let mut delete_source = persistent.delete_source.unwrap_or(false);

	let mut focus = Focus::ArchivePath;
	let mut error_msg: Option<String> = None;

	let formats = ArchiveFormat::all();
	let levels = CompressionLevel::all();
	let methods = CompressionMethod::all();

	let mut format_idx = formats.iter().position(|f| *f == format).unwrap_or(0);
	if level_idx >= levels.len() {
		level_idx = 3;
	}
	if method_idx >= methods.len() {
		method_idx = 0;
	}

	loop {
		let mut layout_rects = Vec::new();

		terminal.draw(|f| {
			let area = f.area();
			let width = (area.width.saturating_sub(4)).min(78).max(50);
			let height = (area.height.saturating_sub(2)).min(24).max(18);

			let popup_area = Rect {
				x: (area.width.saturating_sub(width)) / 2,
				y: (area.height.saturating_sub(height)) / 2,
				width,
				height,
			};

			f.render_widget(Clear, popup_area);

			let title = format!(" Pack files ({}) ", files.len());
			let main_block = Block::default()
				.borders(Borders::ALL)
				.title(title)
				.title_alignment(Alignment::Center)
				.border_style(Style::default().fg(Color::Cyan));
			f.render_widget(main_block, popup_area);

			let inner_area = Rect {
				x: popup_area.x + 2,
				y: popup_area.y + 1,
				width: popup_area.width.saturating_sub(4),
				height: popup_area.height.saturating_sub(2),
			};

			let rows = Layout::default()
				.direction(Direction::Vertical)
				.constraints([
					Constraint::Length(2), // 0: Archive path
					Constraint::Length(2), // 1: Archiver & Level & Method & Solid
					Constraint::Length(4), // 2: Encryption box
					Constraint::Length(2), // 3: Options (delete source)
					Constraint::Length(1), // 4: Error msg
					Constraint::Length(2), // 5: Buttons OK / Cancel
				])
				.split(inner_area);

			layout_rects = rows.to_vec();

			let is_focused = |tgt: &Focus| *tgt == focus;
			let active_style = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);
			let normal_style = Style::default().fg(Color::White);
			let hotkey_style = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);
			let dim_style = Style::default().fg(Color::DarkGray);

			// Row 0: Archive path with visual cursor
			let mut path_spans = vec![Span::styled("Archive path: ", normal_style)];
			path_spans.extend(path_input.render_spans(
				is_focused(&Focus::ArchivePath),
				normal_style,
				active_style,
			));
			f.render_widget(
				Paragraph::new(Line::from(path_spans)).block(
					Block::default()
						.borders(Borders::BOTTOM)
						.border_style(if is_focused(&Focus::ArchivePath) { active_style } else { dim_style }),
				),
				rows[0],
			);

			// Row 1: Compact Archiver & Level & Method & Solid (Dropdown style)
			let row1_spans = vec![
				Span::styled("Archiver", hotkey_style),
				Span::styled(
					format!(":[ {} ]  ", formats[format_idx].label()),
					if is_focused(&Focus::Format) { active_style } else { normal_style },
				),
				Span::styled("Level", hotkey_style),
				Span::styled(
					format!(":[ {} ]  ", levels[level_idx].label()),
					if is_focused(&Focus::Level) { active_style } else { normal_style },
				),
				Span::styled("Method", hotkey_style),
				Span::styled(
					format!(":[ {} ]  ", methods[method_idx].label()),
					if is_focused(&Focus::Method) { active_style } else { normal_style },
				),
				Span::styled("[", dim_style),
				Span::styled(
					if solid { "x" } else { " " },
					if is_focused(&Focus::Solid) { active_style } else { normal_style },
				),
				Span::styled("] ", dim_style),
				Span::styled("Solid", if is_focused(&Focus::Solid) { active_style } else { normal_style }),
			];
			f.render_widget(Paragraph::new(Line::from(row1_spans)), rows[1]);

			// Row 2: Encryption box
			let enc_header = Line::from(vec![
				Span::styled("[", dim_style),
				Span::styled(
					if encrypt { "x" } else { " " },
					if is_focused(&Focus::Encrypt) { active_style } else { normal_style },
				),
				Span::styled("] ", dim_style),
				Span::styled(
					"Encrypt archive",
					if is_focused(&Focus::Encrypt) { active_style } else { normal_style },
				),
				Span::styled("    [", dim_style),
				Span::styled(
					if encrypt_header { "x" } else { " " },
					if is_focused(&Focus::EncryptHeader) { active_style } else { normal_style },
				),
				Span::styled("] ", dim_style),
				Span::styled(
					"Encrypt header (7z)",
					if is_focused(&Focus::EncryptHeader) { active_style } else { normal_style },
				),
				Span::styled("    [", dim_style),
				Span::styled(
					if show_password { "x" } else { " " },
					if is_focused(&Focus::ShowPassword) { active_style } else { normal_style },
				),
				Span::styled("] ", dim_style),
				Span::styled(
					"Show password",
					if is_focused(&Focus::ShowPassword) { active_style } else { normal_style },
				),
			]);

			let mut pwd_spans =
				vec![Span::styled("Password: ", if encrypt { normal_style } else { dim_style })];
			pwd_spans.extend(password_input.render_password_spans(
				is_focused(&Focus::Password),
				show_password,
				if encrypt { normal_style } else { dim_style },
				active_style,
			));
			pwd_spans.push(Span::styled("    Repeat: ", if encrypt { normal_style } else { dim_style }));
			pwd_spans.extend(repeat_input.render_password_spans(
				is_focused(&Focus::RepeatPassword),
				show_password,
				if encrypt { normal_style } else { dim_style },
				active_style,
			));

			let enc_fields = Line::from(pwd_spans);

			let enc_block = Block::default().borders(Borders::ALL).title(" Security ").border_style(
				if is_focused(&Focus::Encrypt)
					|| is_focused(&Focus::EncryptHeader)
					|| is_focused(&Focus::ShowPassword)
					|| is_focused(&Focus::Password)
					|| is_focused(&Focus::RepeatPassword)
				{
					active_style
				} else {
					dim_style
				},
			);
			f.render_widget(Paragraph::new(vec![enc_header, enc_fields]).block(enc_block), rows[2]);

			// Row 3: Options (Delete source)
			let opt_line = Line::from(vec![
				Span::styled("[", dim_style),
				Span::styled(
					if delete_source { "x" } else { " " },
					if is_focused(&Focus::DeleteSource) { active_style } else { normal_style },
				),
				Span::styled("] ", dim_style),
				Span::styled(
					"Delete files after archiving",
					if is_focused(&Focus::DeleteSource) { active_style } else { normal_style },
				),
			]);
			f.render_widget(Paragraph::new(opt_line), rows[3]);

			// Row 4: Error msg
			if let Some(err) = &error_msg {
				f.render_widget(
					Paragraph::new(Line::from(vec![Span::styled(
						err,
						Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
					)])),
					rows[4],
				);
			}

			// Row 5: Buttons { OK } [ Cancel ]
			let btn_line = Line::from(vec![
				Span::styled(
					"  {  OK  }  ",
					if is_focused(&Focus::OkBtn) {
						Style::default().bg(Color::Cyan).fg(Color::Black).add_modifier(Modifier::BOLD)
					} else {
						Style::default().fg(Color::Cyan)
					},
				),
				Span::raw("        "),
				Span::styled(
					"  [ Cancel ]  ",
					if is_focused(&Focus::CancelBtn) {
						Style::default().bg(Color::Red).fg(Color::White).add_modifier(Modifier::BOLD)
					} else {
						Style::default().fg(Color::DarkGray)
					},
				),
			]);
			f.render_widget(Paragraph::new(btn_line).alignment(Alignment::Center), rows[5]);
		})?;

		match event::read()? {
			Event::Mouse(mouse_event) => {
				if mouse_event.kind == MouseEventKind::Down(MouseButton::Left) {
					let mx = mouse_event.column;
					let my = mouse_event.row;

					let in_rect =
						|r: Rect| mx >= r.x && mx < r.x + r.width && my >= r.y && my < r.y + r.height;

					if !layout_rects.is_empty() {
						if in_rect(layout_rects[0]) {
							focus = Focus::ArchivePath;
						} else if in_rect(layout_rects[1]) {
							// Archiver, Level, Method, Solid
							let rel_x = mx.saturating_sub(layout_rects[1].x);
							if rel_x < 18 {
								focus = Focus::Format;
								format_idx = (format_idx + 1) % formats.len();
								format = formats[format_idx];
								update_archive_ext(&mut path_input, format.extension());
							} else if rel_x < 36 {
								focus = Focus::Level;
								level_idx = (level_idx + 1) % levels.len();
							} else if rel_x < 54 {
								focus = Focus::Method;
								method_idx = (method_idx + 1) % methods.len();
							} else {
								focus = Focus::Solid;
								solid = !solid;
							}
						} else if in_rect(layout_rects[2]) {
							// Security block
							let rel_y = my.saturating_sub(layout_rects[2].y);
							let rel_x = mx.saturating_sub(layout_rects[2].x);
							if rel_y <= 1 {
								if rel_x < 22 {
									focus = Focus::Encrypt;
									encrypt = !encrypt;
								} else if rel_x < 48 {
									focus = Focus::EncryptHeader;
									encrypt_header = !encrypt_header;
								} else {
									focus = Focus::ShowPassword;
									show_password = !show_password;
								}
							} else if rel_x < 35 {
								focus = Focus::Password;
							} else {
								focus = Focus::RepeatPassword;
							}
						} else if in_rect(layout_rects[3]) {
							focus = Focus::DeleteSource;
							delete_source = !delete_source;
						} else if in_rect(layout_rects[5]) {
							let rel_x = mx.saturating_sub(layout_rects[5].x);
							let mid = layout_rects[5].width / 2;
							if rel_x < mid {
								focus = Focus::OkBtn;
								// Trigger submit
								if encrypt && password_input.value != repeat_input.value {
									error_msg = Some("Passwords do not match!".to_string());
									focus = Focus::RepeatPassword;
									continue;
								}
								if path_input.value.trim().is_empty() {
									error_msg = Some("Archive path cannot be empty!".to_string());
									focus = Focus::ArchivePath;
									continue;
								}

								save_state(&PersistentArchiveState {
									format: Some(format),
									level_idx: Some(level_idx),
									method_idx: Some(method_idx),
									solid: Some(solid),
									delete_source: Some(delete_source),
									overwrite: None,
								});

								let result = ArchiveResult {
									op: "pack".to_string(),
									format: format.extension().to_string(),
									archive_path: path_input.value.trim().to_string(),
									target_dir: chosen_dir.to_string_lossy().to_string(),
									files,
									level: levels[level_idx].value(),
									method: methods[method_idx].label().to_string(),
									solid,
									password: if encrypt && !password_input.value.is_empty() {
										Some(password_input.value)
									} else {
										None
									},
									encrypt_header: encrypt && encrypt_header && format == ArchiveFormat::SevenZip,
									delete_source,
									overwrite: false,
								};

								let f = fs::File::create("/tmp/che_archive_result.json")?;
								serde_json::to_writer(f, &result)?;
								return Ok(());
							} else {
								return Ok(());
							}
						}
					}
				}
			}
			Event::Key(key) => {
				if key.kind != KeyEventKind::Press {
					continue;
				}

				error_msg = None;

				if key.modifiers.contains(KeyModifiers::CONTROL)
					|| key.modifiers.contains(KeyModifiers::ALT)
				{
					match key.code {
						KeyCode::Left | KeyCode::Char('b') => {
							match focus {
								Focus::ArchivePath => path_input.word_left(),
								Focus::Password => password_input.word_left(),
								Focus::RepeatPassword => repeat_input.word_left(),
								_ => {}
							}
							continue;
						}
						KeyCode::Right | KeyCode::Char('f') => {
							match focus {
								Focus::ArchivePath => path_input.word_right(),
								Focus::Password => password_input.word_right(),
								Focus::RepeatPassword => repeat_input.word_right(),
								_ => {}
							}
							continue;
						}
						KeyCode::Backspace | KeyCode::Char('w') | KeyCode::Char('h') => {
							match focus {
								Focus::ArchivePath => path_input.delete_word_left(),
								Focus::Password => password_input.delete_word_left(),
								Focus::RepeatPassword => repeat_input.delete_word_left(),
								_ => {}
							}
							continue;
						}
						KeyCode::Delete | KeyCode::Char('d') => {
							match focus {
								Focus::ArchivePath => path_input.delete_word_right(),
								Focus::Password => password_input.delete_word_right(),
								Focus::RepeatPassword => repeat_input.delete_word_right(),
								_ => {}
							}
							continue;
						}
						KeyCode::Char('u') => {
							match focus {
								Focus::ArchivePath => path_input.delete_to_start(),
								Focus::Password => password_input.delete_to_start(),
								Focus::RepeatPassword => repeat_input.delete_to_start(),
								_ => {}
							}
							continue;
						}
						KeyCode::Char('k') => {
							match focus {
								Focus::ArchivePath => path_input.delete_to_end(),
								Focus::Password => password_input.delete_to_end(),
								Focus::RepeatPassword => repeat_input.delete_to_end(),
								_ => {}
							}
							continue;
						}
						KeyCode::Char('a') => {
							match focus {
								Focus::ArchivePath => path_input.move_home(),
								Focus::Password => password_input.move_home(),
								Focus::RepeatPassword => repeat_input.move_home(),
								_ => {}
							}
							continue;
						}
						KeyCode::Char('e') => {
							match focus {
								Focus::ArchivePath => path_input.move_end(),
								Focus::Password => password_input.move_end(),
								Focus::RepeatPassword => repeat_input.move_end(),
								_ => {}
							}
							continue;
						}
						_ => {}
					}
				}

				match key.code {
					KeyCode::Esc => return Ok(()),
					KeyCode::Tab => {
						focus = next_pack_focus(&focus);
					}
					KeyCode::BackTab => {
						focus = prev_pack_focus(&focus);
					}
					KeyCode::Home => match focus {
						Focus::ArchivePath => path_input.move_home(),
						Focus::Password => password_input.move_home(),
						Focus::RepeatPassword => repeat_input.move_home(),
						_ => {}
					},
					KeyCode::End => match focus {
						Focus::ArchivePath => path_input.move_end(),
						Focus::Password => password_input.move_end(),
						Focus::RepeatPassword => repeat_input.move_end(),
						_ => {}
					},
					KeyCode::Delete => match focus {
						Focus::ArchivePath => path_input.delete(),
						Focus::Password => password_input.delete(),
						Focus::RepeatPassword => repeat_input.delete(),
						_ => {}
					},
					KeyCode::Enter => match focus {
						Focus::CancelBtn => return Ok(()),
						_ => {
							if encrypt && password_input.value != repeat_input.value {
								error_msg = Some("Passwords do not match!".to_string());
								focus = Focus::RepeatPassword;
								continue;
							}
							if path_input.value.trim().is_empty() {
								error_msg = Some("Archive path cannot be empty!".to_string());
								focus = Focus::ArchivePath;
								continue;
							}

							save_state(&PersistentArchiveState {
								format: Some(format),
								level_idx: Some(level_idx),
								method_idx: Some(method_idx),
								solid: Some(solid),
								delete_source: Some(delete_source),
								overwrite: None,
							});

							let result = ArchiveResult {
								op: "pack".to_string(),
								format: format.extension().to_string(),
								archive_path: path_input.value.trim().to_string(),
								target_dir: chosen_dir.to_string_lossy().to_string(),
								files,
								level: levels[level_idx].value(),
								method: methods[method_idx].label().to_string(),
								solid,
								password: if encrypt && !password_input.value.is_empty() {
									Some(password_input.value)
								} else {
									None
								},
								encrypt_header: encrypt && encrypt_header && format == ArchiveFormat::SevenZip,
								delete_source,
								overwrite: false,
							};

							let f = fs::File::create("/tmp/che_archive_result.json")?;
							serde_json::to_writer(f, &result)?;
							return Ok(());
						}
					},
					KeyCode::Char(' ') => match focus {
						Focus::Format => {
							format_idx = (format_idx + 1) % formats.len();
							format = formats[format_idx];
							update_archive_ext(&mut path_input, format.extension());
						}
						Focus::Level => {
							level_idx = (level_idx + 1) % levels.len();
						}
						Focus::Method => {
							method_idx = (method_idx + 1) % methods.len();
						}
						Focus::Solid => solid = !solid,
						Focus::Encrypt => encrypt = !encrypt,
						Focus::EncryptHeader => encrypt_header = !encrypt_header,
						Focus::ShowPassword => show_password = !show_password,
						Focus::DeleteSource => delete_source = !delete_source,
						Focus::CancelBtn => return Ok(()),
						_ => {}
					},
					KeyCode::Left => match focus {
						Focus::ArchivePath => path_input.move_left(),
						Focus::Password => password_input.move_left(),
						Focus::RepeatPassword => repeat_input.move_left(),
						Focus::Format => {
							format_idx = if format_idx == 0 { formats.len() - 1 } else { format_idx - 1 };
							format = formats[format_idx];
							update_archive_ext(&mut path_input, format.extension());
						}
						Focus::Level => {
							level_idx = if level_idx == 0 { levels.len() - 1 } else { level_idx - 1 };
						}
						Focus::Method => {
							method_idx = if method_idx == 0 { methods.len() - 1 } else { method_idx - 1 };
						}
						Focus::Solid => solid = !solid,
						Focus::Encrypt => encrypt = !encrypt,
						Focus::EncryptHeader => encrypt_header = !encrypt_header,
						Focus::ShowPassword => show_password = !show_password,
						Focus::DeleteSource => delete_source = !delete_source,
						Focus::CancelBtn => focus = Focus::OkBtn,
						Focus::OkBtn => focus = Focus::CancelBtn,
						_ => {}
					},
					KeyCode::Right => match focus {
						Focus::ArchivePath => path_input.move_right(),
						Focus::Password => password_input.move_right(),
						Focus::RepeatPassword => repeat_input.move_right(),
						Focus::Format => {
							format_idx = (format_idx + 1) % formats.len();
							format = formats[format_idx];
							update_archive_ext(&mut path_input, format.extension());
						}
						Focus::Level => {
							level_idx = (level_idx + 1) % levels.len();
						}
						Focus::Method => {
							method_idx = (method_idx + 1) % methods.len();
						}
						Focus::Solid => solid = !solid,
						Focus::Encrypt => encrypt = !encrypt,
						Focus::EncryptHeader => encrypt_header = !encrypt_header,
						Focus::ShowPassword => show_password = !show_password,
						Focus::DeleteSource => delete_source = !delete_source,
						Focus::OkBtn => focus = Focus::CancelBtn,
						Focus::CancelBtn => focus = Focus::OkBtn,
						_ => {}
					},
					KeyCode::Up => {
						focus = prev_pack_focus(&focus);
					}
					KeyCode::Down => {
						focus = next_pack_focus(&focus);
					}
					KeyCode::Backspace => match focus {
						Focus::ArchivePath => path_input.backspace(),
						Focus::Password => password_input.backspace(),
						Focus::RepeatPassword => repeat_input.backspace(),
						_ => {}
					},
					KeyCode::Char(c) => match focus {
						Focus::ArchivePath => path_input.insert(c),
						Focus::Password => {
							encrypt = true;
							password_input.insert(c);
						}
						Focus::RepeatPassword => {
							encrypt = true;
							repeat_input.insert(c);
						}
						_ => match c.to_ascii_lowercase() {
							'a' => focus = Focus::ArchivePath,
							'f' => {
								format_idx = (format_idx + 1) % formats.len();
								format = formats[format_idx];
								update_archive_ext(&mut path_input, format.extension());
								focus = Focus::Format;
							}
							'l' => {
								level_idx = (level_idx + 1) % levels.len();
								focus = Focus::Level;
							}
							'm' => {
								method_idx = (method_idx + 1) % methods.len();
								focus = Focus::Method;
							}
							's' => solid = !solid,
							'e' => encrypt = !encrypt,
							'h' => encrypt_header = !encrypt_header,
							'p' => {
								encrypt = true;
								focus = Focus::Password;
							}
							'd' => delete_source = !delete_source,
							'o' => focus = Focus::OkBtn,
							'c' => focus = Focus::CancelBtn,
							_ => {}
						},
					},
					_ => {}
				}
			}
			_ => {}
		}
	}
}

pub fn update_archive_ext(input: &mut TextInput, new_ext: &str) {
	let path = Path::new(&input.value);
	let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("archive");
	let clean_stem = if file_stem.ends_with(".tar") {
		file_stem.strip_suffix(".tar").unwrap_or(file_stem)
	} else {
		file_stem
	};
	let parent = path.parent().unwrap_or_else(|| Path::new(""));
	let new_filename = format!("{clean_stem}.{new_ext}");
	input.value = parent.join(new_filename).to_string_lossy().to_string();
	input.cursor = input.value.chars().count();
}

fn next_pack_focus(f: &Focus) -> Focus {
	match f {
		Focus::ArchivePath => Focus::Format,
		Focus::Format => Focus::Level,
		Focus::Level => Focus::Method,
		Focus::Method => Focus::Solid,
		Focus::Solid => Focus::Encrypt,
		Focus::Encrypt => Focus::EncryptHeader,
		Focus::EncryptHeader => Focus::ShowPassword,
		Focus::ShowPassword => Focus::Password,
		Focus::Password => Focus::RepeatPassword,
		Focus::RepeatPassword => Focus::DeleteSource,
		Focus::DeleteSource => Focus::OkBtn,
		Focus::OkBtn => Focus::CancelBtn,
		Focus::CancelBtn => Focus::ArchivePath,
		_ => Focus::ArchivePath,
	}
}

fn prev_pack_focus(f: &Focus) -> Focus {
	match f {
		Focus::ArchivePath => Focus::CancelBtn,
		Focus::Format => Focus::ArchivePath,
		Focus::Level => Focus::Format,
		Focus::Method => Focus::Level,
		Focus::Solid => Focus::Method,
		Focus::Encrypt => Focus::Solid,
		Focus::EncryptHeader => Focus::Encrypt,
		Focus::ShowPassword => Focus::EncryptHeader,
		Focus::Password => Focus::ShowPassword,
		Focus::RepeatPassword => Focus::Password,
		Focus::DeleteSource => Focus::RepeatPassword,
		Focus::OkBtn => Focus::DeleteSource,
		Focus::CancelBtn => Focus::OkBtn,
		_ => Focus::ArchivePath,
	}
}

fn run_extract_app(
	terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
	files: Vec<String>,
	mode_arg: String,
	output_dir: Option<String>,
	opposite_dir: Option<String>,
) -> Result<()> {
	let persistent = load_state();

	let archive_path = Path::new(&files[0]);
	let parent_dir = output_dir
		.as_deref()
		.map(Path::new)
		.unwrap_or_else(|| archive_path.parent().unwrap_or(Path::new(".")));

	let stem = archive_path.file_stem().and_then(|s| s.to_str()).unwrap_or("extracted");
	let clean_stem =
		if stem.ends_with(".tar") { stem.strip_suffix(".tar").unwrap_or(stem) } else { stem };

	let current_dir_str = parent_dir.to_string_lossy().to_string();
	let subdir_str = parent_dir.join(clean_stem).to_string_lossy().to_string();
	let opp_dir_str = opposite_dir.unwrap_or_else(|| current_dir_str.clone());

	let initial_dest = if mode_arg == "extract-current" {
		current_dir_str.clone()
	} else if mode_arg == "extract-opposite" {
		opp_dir_str.clone()
	} else {
		subdir_str.clone()
	};

	let mut dest_input = TextInput::new(&initial_dest);
	let mut password_input = TextInput::new("");
	let mut show_password = false;
	let mut overwrite = persistent.overwrite.unwrap_or(true);
	let mut delete_archive = false;

	let mut focus = Focus::DestPath;
	let mut error_msg: Option<String> = None;

	loop {
		let mut layout_rects = Vec::new();

		terminal.draw(|f| {
			let area = f.area();
			let width = (area.width.saturating_sub(4)).min(76).max(50);
			let height = (area.height.saturating_sub(2)).min(20).max(15);

			let popup_area = Rect {
				x: (area.width.saturating_sub(width)) / 2,
				y: (area.height.saturating_sub(height)) / 2,
				width,
				height,
			};

			f.render_widget(Clear, popup_area);

			let title = format!(
				" Extract files: {} ",
				archive_path.file_name().and_then(|s| s.to_str()).unwrap_or("")
			);
			let main_block = Block::default()
				.borders(Borders::ALL)
				.title(title)
				.title_alignment(Alignment::Center)
				.border_style(Style::default().fg(Color::Cyan));
			f.render_widget(main_block, popup_area);

			let inner_area = Rect {
				x: popup_area.x + 2,
				y: popup_area.y + 1,
				width: popup_area.width.saturating_sub(4),
				height: popup_area.height.saturating_sub(2),
			};

			let rows = Layout::default()
				.direction(Direction::Vertical)
				.constraints([
					Constraint::Length(2), // 0: Dest path
					Constraint::Length(2), // 1: Quick targets [ Current ] [ Subdir ] [ Opposite ]
					Constraint::Length(3), // 2: Password
					Constraint::Length(2), // 3: Options (overwrite, delete archive)
					Constraint::Length(1), // 4: Error msg
					Constraint::Length(2), // 5: Buttons OK / Cancel
				])
				.split(inner_area);

			layout_rects = rows.to_vec();

			let is_focused = |tgt: &Focus| *tgt == focus;
			let active_style = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);
			let normal_style = Style::default().fg(Color::White);
			let dim_style = Style::default().fg(Color::DarkGray);

			// Row 0: Dest path with visual cursor
			let mut path_spans = vec![Span::styled("Extract to: ", normal_style)];
			path_spans.extend(dest_input.render_spans(
				is_focused(&Focus::DestPath),
				normal_style,
				active_style,
			));
			f.render_widget(
				Paragraph::new(Line::from(path_spans)).block(
					Block::default().borders(Borders::BOTTOM).border_style(if is_focused(&Focus::DestPath) {
						active_style
					} else {
						dim_style
					}),
				),
				rows[0],
			);

			// Row 1: Quick target buttons
			let quick_line = Line::from(vec![
				Span::styled("Quick destination: ", dim_style),
				Span::styled(
					"[ Current dir ] ",
					if is_focused(&Focus::DestCurrent) {
						active_style
					} else if dest_input.value == current_dir_str {
						Style::default().fg(Color::Cyan)
					} else {
						normal_style
					},
				),
				Span::styled(
					"[ Subfolder ] ",
					if is_focused(&Focus::DestSubdir) {
						active_style
					} else if dest_input.value == subdir_str {
						Style::default().fg(Color::Cyan)
					} else {
						normal_style
					},
				),
				Span::styled(
					"[ Opposite pane ]",
					if is_focused(&Focus::DestOpposite) {
						active_style
					} else if dest_input.value == opp_dir_str {
						Style::default().fg(Color::Cyan)
					} else {
						normal_style
					},
				),
			]);
			f.render_widget(Paragraph::new(quick_line), rows[1]);

			// Row 2: Password with visual cursor
			let mut pwd_spans = vec![Span::styled("Password (if encrypted): ", normal_style)];
			pwd_spans.extend(password_input.render_password_spans(
				is_focused(&Focus::Password),
				show_password,
				normal_style,
				active_style,
			));
			pwd_spans.push(Span::styled("    [", dim_style));
			pwd_spans.push(Span::styled(
				if show_password { "x" } else { " " },
				if is_focused(&Focus::ShowPassword) { active_style } else { normal_style },
			));
			pwd_spans.push(Span::styled("] ", dim_style));
			pwd_spans.push(Span::styled(
				"Show password",
				if is_focused(&Focus::ShowPassword) { active_style } else { normal_style },
			));

			let pwd_block = Block::default().borders(Borders::ALL).title(" Security ").border_style(
				if is_focused(&Focus::Password) || is_focused(&Focus::ShowPassword) {
					active_style
				} else {
					dim_style
				},
			);
			f.render_widget(Paragraph::new(Line::from(pwd_spans)).block(pwd_block), rows[2]);

			// Row 3: Options
			let opt_line = Line::from(vec![
				Span::styled("[", dim_style),
				Span::styled(
					if overwrite { "x" } else { " " },
					if is_focused(&Focus::Overwrite) { active_style } else { normal_style },
				),
				Span::styled("] ", dim_style),
				Span::styled(
					"Overwrite existing files",
					if is_focused(&Focus::Overwrite) { active_style } else { normal_style },
				),
				Span::styled("    [", dim_style),
				Span::styled(
					if delete_archive { "x" } else { " " },
					if is_focused(&Focus::DeleteArchive) { active_style } else { normal_style },
				),
				Span::styled("] ", dim_style),
				Span::styled(
					"Delete archive after extraction",
					if is_focused(&Focus::DeleteArchive) { active_style } else { normal_style },
				),
			]);
			f.render_widget(Paragraph::new(opt_line), rows[3]);

			// Row 4: Error
			if let Some(err) = &error_msg {
				f.render_widget(
					Paragraph::new(Line::from(vec![Span::styled(
						err,
						Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
					)])),
					rows[4],
				);
			}

			// Row 5: Buttons
			let btn_line = Line::from(vec![
				Span::styled(
					"  { Extract }  ",
					if is_focused(&Focus::OkBtn) {
						Style::default().bg(Color::Cyan).fg(Color::Black).add_modifier(Modifier::BOLD)
					} else {
						Style::default().fg(Color::Cyan)
					},
				),
				Span::raw("        "),
				Span::styled(
					"  [ Cancel ]  ",
					if is_focused(&Focus::CancelBtn) {
						Style::default().bg(Color::Red).fg(Color::White).add_modifier(Modifier::BOLD)
					} else {
						Style::default().fg(Color::DarkGray)
					},
				),
			]);
			f.render_widget(Paragraph::new(btn_line).alignment(Alignment::Center), rows[5]);
		})?;

		match event::read()? {
			Event::Mouse(mouse_event) => {
				if mouse_event.kind == MouseEventKind::Down(MouseButton::Left) {
					let mx = mouse_event.column;
					let my = mouse_event.row;

					let in_rect =
						|r: Rect| mx >= r.x && mx < r.x + r.width && my >= r.y && my < r.y + r.height;

					if !layout_rects.is_empty() {
						if in_rect(layout_rects[0]) {
							focus = Focus::DestPath;
						} else if in_rect(layout_rects[1]) {
							let rel_x = mx.saturating_sub(layout_rects[1].x);
							if rel_x < 36 {
								focus = Focus::DestCurrent;
								dest_input.value = current_dir_str.clone();
								dest_input.cursor = dest_input.value.chars().count();
							} else if rel_x < 52 {
								focus = Focus::DestSubdir;
								dest_input.value = subdir_str.clone();
								dest_input.cursor = dest_input.value.chars().count();
							} else {
								focus = Focus::DestOpposite;
								dest_input.value = opp_dir_str.clone();
								dest_input.cursor = dest_input.value.chars().count();
							}
						} else if in_rect(layout_rects[2]) {
							let rel_x = mx.saturating_sub(layout_rects[2].x);
							if rel_x < 42 {
								focus = Focus::Password;
							} else {
								focus = Focus::ShowPassword;
								show_password = !show_password;
							}
						} else if in_rect(layout_rects[3]) {
							let rel_x = mx.saturating_sub(layout_rects[3].x);
							if rel_x < 32 {
								focus = Focus::Overwrite;
								overwrite = !overwrite;
							} else {
								focus = Focus::DeleteArchive;
								delete_archive = !delete_archive;
							}
						} else if in_rect(layout_rects[5]) {
							let rel_x = mx.saturating_sub(layout_rects[5].x);
							let mid = layout_rects[5].width / 2;
							if rel_x < mid {
								if dest_input.value.trim().is_empty() {
									error_msg = Some("Destination directory cannot be empty!".to_string());
									focus = Focus::DestPath;
									continue;
								}

								save_state(&PersistentArchiveState {
									format: None,
									level_idx: None,
									method_idx: None,
									solid: None,
									delete_source: None,
									overwrite: Some(overwrite),
								});

								let result = ArchiveResult {
									op: "extract".to_string(),
									format: "".to_string(),
									archive_path: archive_path.to_string_lossy().to_string(),
									target_dir: dest_input.value.trim().to_string(),
									files,
									level: 0,
									method: "".to_string(),
									solid: false,
									password: if !password_input.value.is_empty() {
										Some(password_input.value)
									} else {
										None
									},
									encrypt_header: false,
									delete_source: delete_archive,
									overwrite,
								};

								let f = fs::File::create("/tmp/che_archive_result.json")?;
								serde_json::to_writer(f, &result)?;
								return Ok(());
							} else {
								return Ok(());
							}
						}
					}
				}
			}
			Event::Key(key) => {
				if key.kind != KeyEventKind::Press {
					continue;
				}

				error_msg = None;

				if key.modifiers.contains(KeyModifiers::CONTROL)
					|| key.modifiers.contains(KeyModifiers::ALT)
				{
					match key.code {
						KeyCode::Left | KeyCode::Char('b') => {
							match focus {
								Focus::DestPath => dest_input.word_left(),
								Focus::Password => password_input.word_left(),
								_ => {}
							}
							continue;
						}
						KeyCode::Right | KeyCode::Char('f') => {
							match focus {
								Focus::DestPath => dest_input.word_right(),
								Focus::Password => password_input.word_right(),
								_ => {}
							}
							continue;
						}
						KeyCode::Backspace | KeyCode::Char('w') | KeyCode::Char('h') => {
							match focus {
								Focus::DestPath => dest_input.delete_word_left(),
								Focus::Password => password_input.delete_word_left(),
								_ => {}
							}
							continue;
						}
						KeyCode::Delete | KeyCode::Char('d') => {
							match focus {
								Focus::DestPath => dest_input.delete_word_right(),
								Focus::Password => password_input.delete_word_right(),
								_ => {}
							}
							continue;
						}
						KeyCode::Char('u') => {
							match focus {
								Focus::DestPath => dest_input.delete_to_start(),
								Focus::Password => password_input.delete_to_start(),
								_ => {}
							}
							continue;
						}
						KeyCode::Char('k') => {
							match focus {
								Focus::DestPath => dest_input.delete_to_end(),
								Focus::Password => password_input.delete_to_end(),
								_ => {}
							}
							continue;
						}
						KeyCode::Char('a') => {
							match focus {
								Focus::DestPath => dest_input.move_home(),
								Focus::Password => password_input.move_home(),
								_ => {}
							}
							continue;
						}
						KeyCode::Char('e') => {
							match focus {
								Focus::DestPath => dest_input.move_end(),
								Focus::Password => password_input.move_end(),
								_ => {}
							}
							continue;
						}
						_ => {}
					}
				}

				match key.code {
					KeyCode::Esc => return Ok(()),
					KeyCode::Tab => {
						focus = next_extract_focus(&focus);
					}
					KeyCode::BackTab => {
						focus = prev_extract_focus(&focus);
					}
					KeyCode::Home => match focus {
						Focus::DestPath => dest_input.move_home(),
						Focus::Password => password_input.move_home(),
						_ => {}
					},
					KeyCode::End => match focus {
						Focus::DestPath => dest_input.move_end(),
						Focus::Password => password_input.move_end(),
						_ => {}
					},
					KeyCode::Delete => match focus {
						Focus::DestPath => dest_input.delete(),
						Focus::Password => password_input.delete(),
						_ => {}
					},
					KeyCode::Enter => match focus {
						Focus::DestCurrent => {
							dest_input.value = current_dir_str.clone();
							dest_input.cursor = dest_input.value.chars().count();
						}
						Focus::DestSubdir => {
							dest_input.value = subdir_str.clone();
							dest_input.cursor = dest_input.value.chars().count();
						}
						Focus::DestOpposite => {
							dest_input.value = opp_dir_str.clone();
							dest_input.cursor = dest_input.value.chars().count();
						}
						Focus::CancelBtn => return Ok(()),
						_ => {
							if dest_input.value.trim().is_empty() {
								error_msg = Some("Destination directory cannot be empty!".to_string());
								focus = Focus::DestPath;
								continue;
							}

							save_state(&PersistentArchiveState {
								format: None,
								level_idx: None,
								method_idx: None,
								solid: None,
								delete_source: None,
								overwrite: Some(overwrite),
							});

							let result = ArchiveResult {
								op: "extract".to_string(),
								format: "".to_string(),
								archive_path: archive_path.to_string_lossy().to_string(),
								target_dir: dest_input.value.trim().to_string(),
								files,
								level: 0,
								method: "".to_string(),
								solid: false,
								password: if !password_input.value.is_empty() {
									Some(password_input.value)
								} else {
									None
								},
								encrypt_header: false,
								delete_source: delete_archive,
								overwrite,
							};

							let f = fs::File::create("/tmp/che_archive_result.json")?;
							serde_json::to_writer(f, &result)?;
							return Ok(());
						}
					},
					KeyCode::Char(' ') => match focus {
						Focus::DestCurrent => {
							dest_input.value = current_dir_str.clone();
							dest_input.cursor = dest_input.value.chars().count();
						}
						Focus::DestSubdir => {
							dest_input.value = subdir_str.clone();
							dest_input.cursor = dest_input.value.chars().count();
						}
						Focus::DestOpposite => {
							dest_input.value = opp_dir_str.clone();
							dest_input.cursor = dest_input.value.chars().count();
						}
						Focus::ShowPassword => show_password = !show_password,
						Focus::Overwrite => overwrite = !overwrite,
						Focus::DeleteArchive => delete_archive = !delete_archive,
						Focus::CancelBtn => return Ok(()),
						_ => {}
					},
					KeyCode::Left => match focus {
						Focus::DestPath => dest_input.move_left(),
						Focus::Password => password_input.move_left(),
						Focus::DestSubdir => focus = Focus::DestCurrent,
						Focus::DestOpposite => focus = Focus::DestSubdir,
						Focus::ShowPassword => show_password = !show_password,
						Focus::Overwrite => overwrite = !overwrite,
						Focus::DeleteArchive => delete_archive = !delete_archive,
						Focus::CancelBtn => focus = Focus::OkBtn,
						Focus::OkBtn => focus = Focus::CancelBtn,
						_ => {}
					},
					KeyCode::Right => match focus {
						Focus::DestPath => dest_input.move_right(),
						Focus::Password => password_input.move_right(),
						Focus::DestCurrent => focus = Focus::DestSubdir,
						Focus::DestSubdir => focus = Focus::DestOpposite,
						Focus::ShowPassword => show_password = !show_password,
						Focus::Overwrite => overwrite = !overwrite,
						Focus::DeleteArchive => delete_archive = !delete_archive,
						Focus::OkBtn => focus = Focus::CancelBtn,
						Focus::CancelBtn => focus = Focus::OkBtn,
						_ => {}
					},
					KeyCode::Up => focus = prev_extract_focus(&focus),
					KeyCode::Down => focus = next_extract_focus(&focus),
					KeyCode::Backspace => match focus {
						Focus::DestPath => dest_input.backspace(),
						Focus::Password => password_input.backspace(),
						_ => {}
					},
					KeyCode::Char(c) => match focus {
						Focus::DestPath => dest_input.insert(c),
						Focus::Password => password_input.insert(c),
						_ => match c.to_ascii_lowercase() {
							'd' => focus = Focus::DestPath,
							'c' => {
								dest_input.value = current_dir_str.clone();
								dest_input.cursor = dest_input.value.chars().count();
							}
							's' => {
								dest_input.value = subdir_str.clone();
								dest_input.cursor = dest_input.value.chars().count();
							}
							'p' => focus = Focus::Password,
							'o' => overwrite = !overwrite,
							'x' => focus = Focus::OkBtn,
							_ => {}
						},
					},
					_ => {}
				}
			}
			_ => {}
		}
	}
}

fn next_extract_focus(f: &Focus) -> Focus {
	match f {
		Focus::DestPath => Focus::DestCurrent,
		Focus::DestCurrent => Focus::DestSubdir,
		Focus::DestSubdir => Focus::DestOpposite,
		Focus::DestOpposite => Focus::Password,
		Focus::Password => Focus::ShowPassword,
		Focus::ShowPassword => Focus::Overwrite,
		Focus::Overwrite => Focus::DeleteArchive,
		Focus::DeleteArchive => Focus::OkBtn,
		Focus::OkBtn => Focus::CancelBtn,
		Focus::CancelBtn => Focus::DestPath,
		_ => Focus::DestPath,
	}
}

fn prev_extract_focus(f: &Focus) -> Focus {
	match f {
		Focus::DestPath => Focus::CancelBtn,
		Focus::DestCurrent => Focus::DestPath,
		Focus::DestSubdir => Focus::DestCurrent,
		Focus::DestOpposite => Focus::DestSubdir,
		Focus::Password => Focus::DestOpposite,
		Focus::ShowPassword => Focus::Password,
		Focus::Overwrite => Focus::ShowPassword,
		Focus::DeleteArchive => Focus::Overwrite,
		Focus::OkBtn => Focus::DeleteArchive,
		Focus::CancelBtn => Focus::OkBtn,
		_ => Focus::DestPath,
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_archive_formats() {
		assert_eq!(ArchiveFormat::SevenZip.extension(), "7z");
		assert_eq!(ArchiveFormat::Zip.extension(), "zip");
		assert_eq!(ArchiveFormat::TarGz.extension(), "tar.gz");
		assert_eq!(ArchiveFormat::TarXz.extension(), "tar.xz");
		assert_eq!(ArchiveFormat::all().len(), 6);
	}

	#[test]
	fn test_compression_levels() {
		assert_eq!(CompressionLevel::Store.value(), 0);
		assert_eq!(CompressionLevel::Normal.value(), 5);
		assert_eq!(CompressionLevel::Ultra.value(), 9);
	}

	#[test]
	fn test_text_input_ops() {
		let mut input = TextInput::new("test string");
		assert_eq!(input.cursor, 11);
		input.word_left();
		assert_eq!(input.cursor, 5);
		input.word_left();
		assert_eq!(input.cursor, 0);
		input.word_right();
		assert_eq!(input.cursor, 5);
		input.move_end();
		assert_eq!(input.cursor, 11);
		input.delete_word_left();
		assert_eq!(input.value, "test ");
		input.move_home();
		input.delete();
		assert_eq!(input.value, "est ");
	}

	#[test]
	fn test_update_ext() {
		let mut input = TextInput::new("archive.zip");
		update_archive_ext(&mut input, "7z");
		assert_eq!(input.value, "archive.7z");

		let mut input_tar = TextInput::new("archive.tar.gz");
		update_archive_ext(&mut input_tar, "tar.xz");
		assert_eq!(input_tar.value, "archive.tar.xz");

		let test_dir = std::path::Path::new("dir").join("archive.zip");
		let mut input_path = TextInput::new(&test_dir.to_string_lossy());
		update_archive_ext(&mut input_path, "7z");
		let expected_dir = std::path::Path::new("dir").join("archive.7z");
		assert_eq!(input_path.value, expected_dir.to_string_lossy().as_ref());
	}

	#[test]
	fn test_is_archive_check() {
		assert!(is_archive_file("package.7z"));
		assert!(is_archive_file("docs.zip"));
		assert!(is_archive_file("source.tar.gz"));
		assert!(is_archive_file("data.tar.xz"));
		assert!(!is_archive_file("document.pdf"));
		assert!(!is_archive_file("image.png"));
	}

	#[test]
	fn test_archive_result_serialization() {
		let res = ArchiveResult {
			op: "pack".to_string(),
			format: "7z".to_string(),
			archive_path: "/tmp/test.7z".to_string(),
			target_dir: "/tmp".to_string(),
			files: vec!["/tmp/file1.txt".to_string()],
			level: 5,
			method: "LZMA2".to_string(),
			solid: true,
			password: Some("secret123".to_string()),
			encrypt_header: true,
			delete_source: false,
			overwrite: false,
		};

		let json_str = serde_json::to_string(&res).expect("serialization failed");
		let deserialized: ArchiveResult =
			serde_json::from_str(&json_str).expect("deserialization failed");
		assert_eq!(res, deserialized);
	}
}
