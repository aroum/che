use crossterm::{
	event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
	execute,
	terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
	Terminal,
	backend::CrosstermBackend,
	layout::{Constraint, Direction, Layout},
	style::{Color, Modifier, Style},
	widgets::{Block, Borders, Paragraph, Row, Table, TableState, Wrap},
};
use regex::Regex;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
struct RenameMap {
	old: String,
	new: String,
}

struct TextInput {
	value: String,
	cursor: usize,
}

impl TextInput {
	fn new(value: &str) -> Self {
		Self { value: value.to_string(), cursor: value.chars().count() }
	}

	fn insert(&mut self, c: char) {
		let char_idx = self.cursor;
		let byte_idx =
			self.value.char_indices().map(|(i, _)| i).nth(char_idx).unwrap_or(self.value.len());
		self.value.insert(byte_idx, c);
		self.cursor += 1;
	}

	fn backspace(&mut self) {
		if self.cursor > 0 {
			self.cursor -= 1;
			let char_idx = self.cursor;
			let byte_idx =
				self.value.char_indices().map(|(i, _)| i).nth(char_idx).unwrap_or(self.value.len());
			self.value.remove(byte_idx);
		}
	}

	fn left(&mut self) {
		if self.cursor > 0 {
			self.cursor -= 1;
		}
	}

	fn right(&mut self) {
		if self.cursor < self.value.chars().count() {
			self.cursor += 1;
		}
	}
}

#[derive(PartialEq, Clone, Copy, Debug)]
enum Focus {
	NameMask,
	NameCase,
	ExtMask,
	ExtCase,
	Find,
	Replace,
	Regex,
	CaseSensitive,
	Replace1x,
	CounterStart,
	CounterStep,
	CounterWidth,
	Table,
	OkBtn,
	CancelBtn,
}

impl Focus {
	fn next(self, counter_used: bool) -> Self {
		let n = match self {
			Self::NameMask => Self::NameCase,
			Self::NameCase => Self::ExtMask,
			Self::ExtMask => Self::ExtCase,
			Self::ExtCase => Self::Find,
			Self::Find => Self::Replace,
			Self::Replace => Self::Regex,
			Self::Regex => Self::CaseSensitive,
			Self::CaseSensitive => Self::Replace1x,
			Self::Replace1x => Self::CounterStart,
			Self::CounterStart => Self::CounterStep,
			Self::CounterStep => Self::CounterWidth,
			Self::CounterWidth => Self::OkBtn,
			Self::OkBtn => Self::CancelBtn,
			Self::CancelBtn => Self::Table,
			Self::Table => Self::NameMask,
		};
		if !counter_used && matches!(n, Self::CounterStart | Self::CounterStep | Self::CounterWidth) {
			n.next(counter_used)
		} else {
			n
		}
	}

	fn prev(self, counter_used: bool) -> Self {
		let p = match self {
			Self::NameMask => Self::Table,
			Self::NameCase => Self::NameMask,
			Self::ExtMask => Self::NameCase,
			Self::ExtCase => Self::ExtMask,
			Self::Find => Self::ExtCase,
			Self::Replace => Self::Find,
			Self::Regex => Self::Replace,
			Self::CaseSensitive => Self::Regex,
			Self::Replace1x => Self::CaseSensitive,
			Self::CounterStart => Self::Replace1x,
			Self::CounterStep => Self::CounterStart,
			Self::CounterWidth => Self::CounterStep,
			Self::OkBtn => Self::CounterWidth,
			Self::CancelBtn => Self::OkBtn,
			Self::Table => Self::CancelBtn,
		};
		if !counter_used && matches!(p, Self::CounterStart | Self::CounterStep | Self::CounterWidth) {
			p.prev(counter_used)
		} else {
			p
		}
	}
}

#[derive(PartialEq, Clone, Copy, Debug)]
enum CaseConv {
	NoChange,
	Uppercase,
	Lowercase,
	FirstLetter,
	TitleCase,
}

struct Layouts {
	chunks: Vec<ratatui::layout::Rect>,
	left_chunks: Vec<ratatui::layout::Rect>,
	name_mask_inner: Vec<ratatui::layout::Rect>,
	ext_mask_inner: Vec<ratatui::layout::Rect>,
	fr_inner: Vec<ratatui::layout::Rect>,
	cb_chunks: Vec<ratatui::layout::Rect>,
	counter_chunks: Vec<ratatui::layout::Rect>,
	button_chunks: Vec<ratatui::layout::Rect>,
}

fn calculate_layouts(area: ratatui::layout::Rect) -> Layouts {
	let chunks = Layout::default()
		.direction(Direction::Horizontal)
		.constraints([Constraint::Percentage(42), Constraint::Percentage(58)])
		.split(area)
		.to_vec();

	let left_chunks = Layout::default()
		.direction(Direction::Vertical)
		.constraints([
			Constraint::Length(8),  // Name Mask Box
			Constraint::Length(8),  // Extension Mask Box
			Constraint::Length(10), // Find & Replace Box
			Constraint::Length(5),  // Counter Box
			Constraint::Min(5),     // Legend Box
			Constraint::Length(3),  // Buttons
		])
		.split(chunks[0])
		.to_vec();

	let name_mask_inner = Layout::default()
		.direction(Direction::Vertical)
		.constraints([Constraint::Length(3), Constraint::Length(3)])
		.margin(1)
		.split(left_chunks[0])
		.to_vec();

	let ext_mask_inner = Layout::default()
		.direction(Direction::Vertical)
		.constraints([Constraint::Length(3), Constraint::Length(3)])
		.margin(1)
		.split(left_chunks[1])
		.to_vec();

	let fr_inner = Layout::default()
		.direction(Direction::Vertical)
		.constraints([Constraint::Length(3), Constraint::Length(3), Constraint::Length(2)])
		.margin(1)
		.split(left_chunks[2])
		.to_vec();

	let cb_chunks = Layout::default()
		.direction(Direction::Horizontal)
		.constraints([
			Constraint::Percentage(33),
			Constraint::Percentage(33),
			Constraint::Percentage(34),
		])
		.split(fr_inner[2])
		.to_vec();

	let counter_chunks = Layout::default()
		.direction(Direction::Horizontal)
		.constraints([
			Constraint::Percentage(33),
			Constraint::Percentage(33),
			Constraint::Percentage(34),
		])
		.margin(1)
		.split(left_chunks[3])
		.to_vec();

	let button_chunks = Layout::default()
		.direction(Direction::Horizontal)
		.constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
		.split(left_chunks[5])
		.to_vec();

	Layouts {
		chunks,
		left_chunks,
		name_mask_inner,
		ext_mask_inner,
		fr_inner,
		cb_chunks,
		counter_chunks,
		button_chunks,
	}
}

fn is_counter_used(mask: &str, ext_mask: &str, find: &str, replace: &str) -> bool {
	let check = |s: &str| s.to_uppercase().contains("[C");
	check(mask) || check(ext_mask) || check(find) || check(replace)
}

pub fn run(files: Vec<String>) -> anyhow::Result<()> {
	if files.is_empty() {
		return Ok(());
	}

	enable_raw_mode()?;
	let mut stdout = std::io::stdout();
	execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
	let backend = CrosstermBackend::new(stdout);
	let mut terminal = Terminal::new(backend)?;

	let app_result = run_app(&mut terminal, files);

	disable_raw_mode()?;
	execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
	terminal.show_cursor()?;

	app_result
}

fn run_app(
	terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
	files: Vec<String>,
) -> anyhow::Result<()> {
	let mut mask_input = TextInput::new("[N]");
	let mut ext_input = TextInput::new("[E]");
	let mut find_input = TextInput::new("");
	let mut replace_input = TextInput::new("");
	let mut use_regex = false;
	let mut case_sensitive = false;
	let mut replace_1x = false;
	let mut name_case = CaseConv::NoChange;
	let mut ext_case = CaseConv::NoChange;
	let mut counter_start = TextInput::new("1");
	let mut counter_step = TextInput::new("1");
	let mut counter_width = TextInput::new("3");

	let mut focus = Focus::NameMask;
	let mut table_state = TableState::default();
	if !files.is_empty() {
		table_state.select(Some(0));
	}

	loop {
		let counter_used =
			is_counter_used(&mask_input.value, &ext_input.value, &find_input.value, &replace_input.value);

		let previews = generate_previews(
			&files,
			&mask_input.value,
			&ext_input.value,
			&find_input.value,
			&replace_input.value,
			use_regex,
			case_sensitive,
			replace_1x,
			name_case,
			ext_case,
			&counter_start.value,
			&counter_step.value,
			&counter_width.value,
		);

		terminal.draw(|f| {
			let layouts = calculate_layouts(f.area());

			// Helper to get border style based on active group
			let group_border_style = |targets: &[Focus]| {
				if targets.contains(&focus) {
					Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
				} else {
					Style::default().fg(Color::DarkGray)
				}
			};

			// Helper for active item styling inside group
			let item_border_style = |target: Focus| {
				if focus == target {
					Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
				} else {
					Style::default().fg(Color::DarkGray)
				}
			};

			let item_style = |target: Focus| {
				if focus == target {
					Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
				} else {
					Style::default()
				}
			};

			// 1. Render Name Mask Box
			let name_mask_block = Block::default()
				.borders(Borders::ALL)
				.title(" Name Mask ")
				.border_style(group_border_style(&[Focus::NameMask, Focus::NameCase]));
			let name_mask_rect = layouts.left_chunks[0];
			f.render_widget(name_mask_block, name_mask_rect);

			// Inner Mask Frame
			let mask_frame = Block::default()
				.borders(Borders::ALL)
				.title(" Mask ")
				.border_style(item_border_style(Focus::NameMask));
			f.render_widget(
				Paragraph::new(mask_input.value.as_str()).block(mask_frame),
				layouts.name_mask_inner[0],
			);

			// Inner Case Frame
			let name_case_str = match name_case {
				CaseConv::NoChange => "No change",
				CaseConv::Uppercase => "UPPERCASE",
				CaseConv::Lowercase => "lowercase",
				CaseConv::FirstLetter => "First letter uppercase",
				CaseConv::TitleCase => "Title Case",
			};
			let name_case_frame = Block::default()
				.borders(Borders::ALL)
				.title(" Case ")
				.border_style(item_border_style(Focus::NameCase));
			f.render_widget(
				Paragraph::new(name_case_str).block(name_case_frame),
				layouts.name_mask_inner[1],
			);

			// 2. Render Extension Mask Box
			let ext_mask_block = Block::default()
				.borders(Borders::ALL)
				.title(" Extension Mask ")
				.border_style(group_border_style(&[Focus::ExtMask, Focus::ExtCase]));
			let ext_mask_rect = layouts.left_chunks[1];
			f.render_widget(ext_mask_block, ext_mask_rect);

			// Inner Mask Frame
			let ext_mask_frame = Block::default()
				.borders(Borders::ALL)
				.title(" Mask ")
				.border_style(item_border_style(Focus::ExtMask));
			f.render_widget(
				Paragraph::new(ext_input.value.as_str()).block(ext_mask_frame),
				layouts.ext_mask_inner[0],
			);

			// Inner Case Frame
			let ext_case_str = match ext_case {
				CaseConv::NoChange => "No change",
				CaseConv::Uppercase => "UPPERCASE",
				CaseConv::Lowercase => "lowercase",
				CaseConv::FirstLetter => "First letter uppercase",
				CaseConv::TitleCase => "Title Case",
			};
			let ext_case_frame = Block::default()
				.borders(Borders::ALL)
				.title(" Case ")
				.border_style(item_border_style(Focus::ExtCase));
			f.render_widget(
				Paragraph::new(ext_case_str).block(ext_case_frame),
				layouts.ext_mask_inner[1],
			);

			// 3. Render Find & Replace Box
			let find_replace_block = Block::default()
				.borders(Borders::ALL)
				.title(" Find & Replace ")
				.border_style(group_border_style(&[
					Focus::Find,
					Focus::Replace,
					Focus::Regex,
					Focus::CaseSensitive,
					Focus::Replace1x,
				]));
			let find_replace_rect = layouts.left_chunks[2];
			f.render_widget(find_replace_block, find_replace_rect);

			// Inner Find Frame
			let find_frame = Block::default()
				.borders(Borders::ALL)
				.title(" Find ")
				.border_style(item_border_style(Focus::Find));
			f.render_widget(
				Paragraph::new(find_input.value.as_str()).block(find_frame),
				layouts.fr_inner[0],
			);

			// Inner Replace Frame
			let replace_frame = Block::default()
				.borders(Borders::ALL)
				.title(" Replace ")
				.border_style(item_border_style(Focus::Replace));
			f.render_widget(
				Paragraph::new(replace_input.value.as_str()).block(replace_frame),
				layouts.fr_inner[1],
			);

			// Checkboxes on one line
			let regex_checkbox = format!(" [{}] RegEx", if use_regex { "X" } else { " " });
			f.render_widget(
				Paragraph::new(regex_checkbox).style(item_style(Focus::Regex)),
				layouts.cb_chunks[0],
			);

			let cs_checkbox = format!(" [{}] Case Sens", if case_sensitive { "X" } else { " " });
			f.render_widget(
				Paragraph::new(cs_checkbox).style(item_style(Focus::CaseSensitive)),
				layouts.cb_chunks[1],
			);

			let r1x_checkbox = format!(" [{}] 1x", if replace_1x { "X" } else { " " });
			f.render_widget(
				Paragraph::new(r1x_checkbox).style(item_style(Focus::Replace1x)),
				layouts.cb_chunks[2],
			);

			// 4. Render Counter Box
			let counter_style = if counter_used {
				group_border_style(&[Focus::CounterStart, Focus::CounterStep, Focus::CounterWidth])
			} else {
				Style::default().fg(Color::DarkGray)
			};
			let counter_block =
				Block::default().borders(Borders::ALL).title(" Counter ").border_style(counter_style);
			let counter_rect = layouts.left_chunks[3];
			f.render_widget(counter_block, counter_rect);

			let c_item_border_style = |target: Focus| {
				if !counter_used {
					Style::default().fg(Color::DarkGray)
				} else if focus == target {
					Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
				} else {
					Style::default().fg(Color::DarkGray)
				}
			};
			let c_item_style = |target: Focus| {
				if !counter_used {
					Style::default().fg(Color::DarkGray)
				} else if focus == target {
					Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
				} else {
					Style::default()
				}
			};

			let c_start_frame = Block::default()
				.borders(Borders::ALL)
				.title(" Start ")
				.border_style(c_item_border_style(Focus::CounterStart));
			f.render_widget(
				Paragraph::new(counter_start.value.as_str())
					.block(c_start_frame)
					.style(c_item_style(Focus::CounterStart)),
				layouts.counter_chunks[0],
			);

			let c_step_frame = Block::default()
				.borders(Borders::ALL)
				.title(" Step ")
				.border_style(c_item_border_style(Focus::CounterStep));
			f.render_widget(
				Paragraph::new(counter_step.value.as_str())
					.block(c_step_frame)
					.style(c_item_style(Focus::CounterStep)),
				layouts.counter_chunks[1],
			);

			let c_width_frame = Block::default()
				.borders(Borders::ALL)
				.title(" Width ")
				.border_style(c_item_border_style(Focus::CounterWidth));
			f.render_widget(
				Paragraph::new(counter_width.value.as_str())
					.block(c_width_frame)
					.style(c_item_style(Focus::CounterWidth)),
				layouts.counter_chunks[2],
			);

			// 5. Render Legend / Info Box
			let info_block = Block::default()
				.borders(Borders::ALL)
				.title(" Legend & Placeholders ")
				.border_style(Style::default().fg(Color::DarkGray));
			let info_text = vec![
				ratatui::text::Line::from(
					" [N] Name   [E] Ext   [C] Counter  [Y]/[M]/[D] Date  [h]/[m]/[s] Time",
				),
				ratatui::text::Line::from(
					" [N2-5] Chars 2-5   [N-8-5] Chars 8 to 5 from end   [2-5] Chars 2-5 of full path",
				),
				ratatui::text::Line::from(" [P] Parent dir     [G] Grandparent dir   [[] [   []] ]"),
				ratatui::text::Line::from(
					" [U] Uppercase      [L] Lowercase         [F] First letter uppercase   [n] Reset case",
				),
				ratatui::text::Line::from(" [C10+5:3] Counter start 10, step 5, width 3"),
			];
			// Added word wrap
			f.render_widget(
				Paragraph::new(info_text).block(info_block).wrap(Wrap { trim: true }),
				layouts.left_chunks[4],
			);

			// Render OK / Cancel buttons
			let button_chunks = layouts.button_chunks;
			let ok_style = if focus == Focus::OkBtn {
				Style::default().bg(Color::Green).fg(Color::Black).add_modifier(Modifier::BOLD)
			} else {
				Style::default().fg(Color::Green)
			};
			let ok_p = Paragraph::new(" [ [O]K ] ")
				.alignment(ratatui::layout::Alignment::Center)
				.block(Block::default().borders(Borders::ALL).border_style(item_style(Focus::OkBtn)));
			f.render_widget(ok_p.style(ok_style), button_chunks[0]);

			let cancel_style = if focus == Focus::CancelBtn {
				Style::default().bg(Color::Red).fg(Color::Black).add_modifier(Modifier::BOLD)
			} else {
				Style::default().fg(Color::Red)
			};
			let cancel_p = Paragraph::new(" [ [C]ancel ] ")
				.alignment(ratatui::layout::Alignment::Center)
				.block(Block::default().borders(Borders::ALL).border_style(item_style(Focus::CancelBtn)));
			f.render_widget(cancel_p.style(cancel_style), button_chunks[1]);

			// Right panel (preview table)
			let table_block = Block::default()
				.borders(Borders::ALL)
				.title(" Preview Changes ")
				.border_style(item_style(Focus::Table));

			let rows: Vec<Row> = previews
				.iter()
				.map(|(old, new)| Row::new(vec![cell_from_str(old), cell_from_str(new)]))
				.collect();

			let table = Table::new(rows, [Constraint::Percentage(50), Constraint::Percentage(50)])
				.header(
					Row::new(vec!["Original Name", "New Name"])
						.style(Style::default().add_modifier(Modifier::BOLD)),
				)
				.block(table_block)
				.row_highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD));

			f.render_stateful_widget(table, layouts.chunks[1], &mut table_state);

			// Set text cursor for active input field
			let active_cursor = match focus {
				Focus::NameMask => Some((mask_input.cursor, layouts.name_mask_inner[0])),
				Focus::ExtMask => Some((ext_input.cursor, layouts.ext_mask_inner[0])),
				Focus::Find => Some((find_input.cursor, layouts.fr_inner[0])),
				Focus::Replace => Some((replace_input.cursor, layouts.fr_inner[1])),
				Focus::CounterStart if counter_used => {
					Some((counter_start.cursor, layouts.counter_chunks[0]))
				}
				Focus::CounterStep if counter_used => {
					Some((counter_step.cursor, layouts.counter_chunks[1]))
				}
				Focus::CounterWidth if counter_used => {
					Some((counter_width.cursor, layouts.counter_chunks[2]))
				}
				_ => None,
			};

			if let Some((cursor_pos, rect)) = active_cursor {
				f.set_cursor_position((rect.x + 1 + cursor_pos as u16, rect.y + 1));
			}
		})?;

		if let Event::Key(key) = event::read()? {
			if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('q') {
				return Ok(());
			}

			if key.modifiers.contains(KeyModifiers::ALT) {
				match key.code {
					KeyCode::Char('o' | 'O') => return accept_rename(files, previews),
					KeyCode::Char('c' | 'C') => return Ok(()),
					_ => {}
				}
			}

			match key.code {
				KeyCode::Esc => {
					return Ok(());
				}
				KeyCode::Tab => {
					focus = focus.next(counter_used);
				}
				KeyCode::BackTab => {
					focus = focus.prev(counter_used);
				}
				KeyCode::Up => {
					if focus == Focus::Table {
						let i = match table_state.selected() {
							Some(i) => {
								if i > 0 {
									i - 1
								} else {
									0
								}
							}
							None => 0,
						};
						table_state.select(Some(i));
					} else {
						focus = focus.prev(counter_used);
					}
				}
				KeyCode::Down => {
					if focus == Focus::Table {
						let i = match table_state.selected() {
							Some(i) => {
								if i < files.len() - 1 {
									i + 1
								} else {
									files.len() - 1
								}
							}
							None => 0,
						};
						table_state.select(Some(i));
					} else {
						focus = focus.next(counter_used);
					}
				}
				KeyCode::Left => match focus {
					Focus::NameMask => mask_input.left(),
					Focus::ExtMask => ext_input.left(),
					Focus::Find => find_input.left(),
					Focus::Replace => replace_input.left(),
					Focus::CounterStart if counter_used => counter_start.left(),
					Focus::CounterStep if counter_used => counter_step.left(),
					Focus::CounterWidth if counter_used => counter_width.left(),
					Focus::OkBtn | Focus::CancelBtn => {
						focus = Focus::OkBtn;
					}
					_ => {}
				},
				KeyCode::Right => match focus {
					Focus::NameMask => mask_input.right(),
					Focus::ExtMask => ext_input.right(),
					Focus::Find => find_input.right(),
					Focus::Replace => replace_input.right(),
					Focus::CounterStart if counter_used => counter_start.right(),
					Focus::CounterStep if counter_used => counter_step.right(),
					Focus::CounterWidth if counter_used => counter_width.right(),
					Focus::OkBtn | Focus::CancelBtn => {
						focus = Focus::CancelBtn;
					}
					_ => {}
				},
				KeyCode::Char(' ') | KeyCode::Enter if focus == Focus::Regex => {
					use_regex = !use_regex;
				}
				KeyCode::Char(' ') | KeyCode::Enter if focus == Focus::CaseSensitive => {
					case_sensitive = !case_sensitive;
				}
				KeyCode::Char(' ') | KeyCode::Enter if focus == Focus::Replace1x => {
					replace_1x = !replace_1x;
				}
				KeyCode::Char(' ') | KeyCode::Enter if focus == Focus::NameCase => {
					name_case = match name_case {
						CaseConv::NoChange => CaseConv::Uppercase,
						CaseConv::Uppercase => CaseConv::Lowercase,
						CaseConv::Lowercase => CaseConv::FirstLetter,
						CaseConv::FirstLetter => CaseConv::TitleCase,
						CaseConv::TitleCase => CaseConv::NoChange,
					};
				}
				KeyCode::Char(' ') | KeyCode::Enter if focus == Focus::ExtCase => {
					ext_case = match ext_case {
						CaseConv::NoChange => CaseConv::Uppercase,
						CaseConv::Uppercase => CaseConv::Lowercase,
						CaseConv::Lowercase => CaseConv::FirstLetter,
						CaseConv::FirstLetter => CaseConv::TitleCase,
						CaseConv::TitleCase => CaseConv::NoChange,
					};
				}
				KeyCode::Char(c) => match focus {
					Focus::NameMask => mask_input.insert(c),
					Focus::ExtMask => ext_input.insert(c),
					Focus::Find => find_input.insert(c),
					Focus::Replace => replace_input.insert(c),
					Focus::CounterStart if counter_used => {
						if c.is_ascii_digit() {
							counter_start.insert(c);
						}
					}
					Focus::CounterStep if counter_used => {
						if c.is_ascii_digit() {
							counter_step.insert(c);
						}
					}
					Focus::CounterWidth if counter_used => {
						if c.is_ascii_digit() {
							counter_width.insert(c);
						}
					}
					Focus::OkBtn => {
						if c == '\n' || c == '\r' {
							return accept_rename(files, previews);
						}
					}
					Focus::CancelBtn => {
						if c == '\n' || c == '\r' {
							return Ok(());
						}
					}
					_ => {}
				},
				KeyCode::Backspace => match focus {
					Focus::NameMask => mask_input.backspace(),
					Focus::ExtMask => ext_input.backspace(),
					Focus::Find => find_input.backspace(),
					Focus::Replace => replace_input.backspace(),
					Focus::CounterStart if counter_used => counter_start.backspace(),
					Focus::CounterStep if counter_used => counter_step.backspace(),
					Focus::CounterWidth if counter_used => counter_width.backspace(),
					_ => {}
				},
				KeyCode::Enter => match focus {
					Focus::OkBtn => {
						return accept_rename(files, previews);
					}
					Focus::CancelBtn => {
						return Ok(());
					}
					_ => {
						return accept_rename(files, previews);
					}
				},
				_ => {}
			}
		} else if let Event::Mouse(mouse_event) = event::read()? {
			// Mouse Click Handling
			if mouse_event.kind == event::MouseEventKind::Down(event::MouseButton::Left) {
				let mx = mouse_event.column;
				let my = mouse_event.row;

				let in_rect = |r: ratatui::layout::Rect| {
					mx >= r.x && mx < r.x + r.width && my >= r.y && my < r.y + r.height
				};

				let size = terminal.size()?;
				let area = ratatui::layout::Rect::new(0, 0, size.width, size.height);
				let layouts = calculate_layouts(area);

				// Check Name Mask
				if in_rect(layouts.name_mask_inner[0]) {
					focus = Focus::NameMask;
				} else if in_rect(layouts.name_mask_inner[1]) {
					focus = Focus::NameCase;
					name_case = match name_case {
						CaseConv::NoChange => CaseConv::Uppercase,
						CaseConv::Uppercase => CaseConv::Lowercase,
						CaseConv::Lowercase => CaseConv::FirstLetter,
						CaseConv::FirstLetter => CaseConv::TitleCase,
						CaseConv::TitleCase => CaseConv::NoChange,
					};
				}
				// Check Extension Mask
				else if in_rect(layouts.ext_mask_inner[0]) {
					focus = Focus::ExtMask;
				} else if in_rect(layouts.ext_mask_inner[1]) {
					focus = Focus::ExtCase;
					ext_case = match ext_case {
						CaseConv::NoChange => CaseConv::Uppercase,
						CaseConv::Uppercase => CaseConv::Lowercase,
						CaseConv::Lowercase => CaseConv::FirstLetter,
						CaseConv::FirstLetter => CaseConv::TitleCase,
						CaseConv::TitleCase => CaseConv::NoChange,
					};
				}
				// Check Find & Replace
				else if in_rect(layouts.fr_inner[0]) {
					focus = Focus::Find;
				} else if in_rect(layouts.fr_inner[1]) {
					focus = Focus::Replace;
				}
				// Checkboxes
				else if in_rect(layouts.cb_chunks[0]) {
					focus = Focus::Regex;
					use_regex = !use_regex;
				} else if in_rect(layouts.cb_chunks[1]) {
					focus = Focus::CaseSensitive;
					case_sensitive = !case_sensitive;
				} else if in_rect(layouts.cb_chunks[2]) {
					focus = Focus::Replace1x;
					replace_1x = !replace_1x;
				}
				// Counter (only if counter is used/active)
				else if counter_used && in_rect(layouts.counter_chunks[0]) {
					focus = Focus::CounterStart;
				} else if counter_used && in_rect(layouts.counter_chunks[1]) {
					focus = Focus::CounterStep;
				} else if counter_used && in_rect(layouts.counter_chunks[2]) {
					focus = Focus::CounterWidth;
				}
				// Buttons
				else if in_rect(layouts.button_chunks[0]) {
					return accept_rename(files, previews);
				} else if in_rect(layouts.button_chunks[1]) {
					return Ok(());
				}
				// Table
				else if in_rect(layouts.chunks[1]) {
					focus = Focus::Table;
					let table_y = layouts.chunks[1].y;
					if my >= table_y + 2 && my < table_y + 2 + files.len() as u16 {
						let clicked_row = (my - table_y - 2) as usize;
						table_state.select(Some(clicked_row));
					}
				}
			}
		}
	}
}

// Simple cell converter helper since ratatui cell API version mismatch can happen
fn cell_from_str(s: &str) -> ratatui::widgets::Cell<'_> {
	ratatui::widgets::Cell::from(s.to_string())
}

fn accept_rename(files: Vec<String>, previews: Vec<(String, String)>) -> anyhow::Result<()> {
	let mut rename_map = Vec::new();
	for (old_path, (_, new_name)) in files.into_iter().zip(previews) {
		let parent = Path::new(&old_path).parent().unwrap_or_else(|| Path::new(""));
		let new_path = parent.join(new_name).to_string_lossy().to_string();
		rename_map.push(RenameMap { old: old_path, new: new_path });
	}

	let result_path = "/tmp/che_multirename_result.json";
	let f = std::fs::File::create(result_path)?;
	serde_json::to_writer(f, &rename_map)?;

	Ok(())
}

fn apply_case_conv(s: &str, conv: CaseConv) -> String {
	match conv {
		CaseConv::NoChange => s.to_string(),
		CaseConv::Uppercase => s.to_uppercase(),
		CaseConv::Lowercase => s.to_lowercase(),
		CaseConv::FirstLetter => {
			let mut chars = s.chars();
			match chars.next() {
				None => String::new(),
				Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
			}
		}
		CaseConv::TitleCase => {
			let mut result = String::new();
			let mut capitalize_next = true;
			for c in s.chars() {
				if c.is_alphanumeric() {
					if capitalize_next {
						result.extend(c.to_uppercase());
						capitalize_next = false;
					} else {
						result.extend(c.to_lowercase());
					}
				} else {
					result.push(c);
					capitalize_next = true;
				}
			}
			result
		}
	}
}

#[derive(Clone, Copy)]
enum CaseMode {
	NoChange,
	Uppercase,
	Lowercase,
	TitleCase,
}

fn parse_mask(
	mask: &str,
	name_without_ext: &str,
	ext: &str,
	parent_dir: &str,
	grandparent_dir: &str,
	file_idx: usize,
	c_start: usize,
	c_step: usize,
	c_width: usize,
) -> String {
	let mut result = String::new();
	let mut current_case = CaseMode::NoChange;

	let chars: Vec<char> = mask.chars().collect();
	let mut i = 0;

	while i < chars.len() {
		if chars[i] == '[' {
			// [[] -> [
			if i + 2 < chars.len() && chars[i + 1] == '[' && chars[i + 2] == ']' {
				result.push_str(&apply_case_conv_mode("[", current_case));
				i += 3;
				continue;
			}
			// []] -> ]
			if i + 2 < chars.len() && chars[i + 1] == ']' && chars[i + 2] == ']' {
				result.push_str(&apply_case_conv_mode("]", current_case));
				i += 3;
				continue;
			}

			// Find matching ']'
			let mut close_pos = None;
			for j in i + 1..chars.len() {
				if chars[j] == ']' {
					close_pos = Some(j);
					break;
				}
			}

			if let Some(j) = close_pos {
				let content: String = chars[i + 1..j].iter().collect();
				i = j + 1;

				if content == "U" {
					current_case = CaseMode::Uppercase;
					continue;
				}
				if content == "L" {
					current_case = CaseMode::Lowercase;
					continue;
				}
				if content == "F" {
					current_case = CaseMode::TitleCase;
					continue;
				}
				if content == "n" {
					current_case = CaseMode::NoChange;
					continue;
				}

				// Counter parsing [C...]
				if content.starts_with('C') {
					let re_c = Regex::new(r"^C(\d+)?(?:\+(\d+))?(?::(\d+))?$").unwrap();
					if let Some(caps) = re_c.captures(&content) {
						let f_start = caps
							.get(1)
							.map(|m| m.as_str().parse::<usize>().unwrap_or(c_start))
							.unwrap_or(c_start);
						let f_step =
							caps.get(2).map(|m| m.as_str().parse::<usize>().unwrap_or(c_step)).unwrap_or(c_step);
						let f_width = caps
							.get(3)
							.map(|m| m.as_str().parse::<usize>().unwrap_or(c_width))
							.unwrap_or(c_width);

						let file_c = f_start + file_idx * f_step;
						let counter_str = format!("{:0width$}", file_c, width = f_width);
						result.push_str(&apply_case_conv_mode(&counter_str, current_case));
						continue;
					}
				}

				// Date & Time
				let date_time = chrono::Local::now();
				match content.as_str() {
					"Y" => {
						result
							.push_str(&apply_case_conv_mode(&date_time.format("%Y").to_string(), current_case));
						continue;
					}
					"M" => {
						result
							.push_str(&apply_case_conv_mode(&date_time.format("%m").to_string(), current_case));
						continue;
					}
					"D" => {
						result
							.push_str(&apply_case_conv_mode(&date_time.format("%d").to_string(), current_case));
						continue;
					}
					"h" => {
						result
							.push_str(&apply_case_conv_mode(&date_time.format("%H").to_string(), current_case));
						continue;
					}
					"m" => {
						result
							.push_str(&apply_case_conv_mode(&date_time.format("%M").to_string(), current_case));
						continue;
					}
					"s" => {
						result
							.push_str(&apply_case_conv_mode(&date_time.format("%S").to_string(), current_case));
						continue;
					}
					"d" => {
						result.push_str(&apply_case_conv_mode(
							&date_time.format("%Y-%m-%d").to_string(),
							current_case,
						));
						continue;
					}
					"t" => {
						result.push_str(&apply_case_conv_mode(
							&date_time.format("%H.%M.%S").to_string(),
							current_case,
						));
						continue;
					}
					_ => {}
				}

				// Slices parsing
				let first_char = content.chars().next();
				let slice_part = if first_char.is_some_and(|c| c == 'N' || c == 'E' || c == 'P' || c == 'G')
				{
					&content[1..]
				} else {
					&content[..]
				};

				let full_name_holder;
				let target_str = match first_char {
					Some('N') => name_without_ext,
					Some('E') => ext,
					Some('P') => parent_dir,
					Some('G') => grandparent_dir,
					_ => {
						full_name_holder = if !ext.is_empty() {
							format!("{}.{}", name_without_ext, ext)
						} else {
							name_without_ext.to_string()
						};
						&full_name_holder
					}
				};

				let val = if first_char.is_none()
					&& !content.chars().any(|c| c.is_ascii_digit() || c == '-' || c == ',')
				{
					format!("[{}]", content)
				} else {
					parse_and_extract_slice(target_str, slice_part)
				};

				result.push_str(&apply_case_conv_mode(&val, current_case));
			} else {
				result.push_str(&apply_case_conv_mode("[", current_case));
				i += 1;
			}
		} else {
			result.push_str(&apply_case_conv_mode(&chars[i].to_string(), current_case));
			i += 1;
		}
	}

	result
}

fn apply_case_conv_mode(s: &str, mode: CaseMode) -> String {
	match mode {
		CaseMode::NoChange => s.to_string(),
		CaseMode::Uppercase => s.to_uppercase(),
		CaseMode::Lowercase => s.to_lowercase(),
		CaseMode::TitleCase => {
			let mut result = String::new();
			let mut capitalize_next = true;
			for c in s.chars() {
				if c.is_alphanumeric() {
					if capitalize_next {
						result.extend(c.to_uppercase());
						capitalize_next = false;
					} else {
						result.extend(c.to_lowercase());
					}
				} else {
					result.push(c);
					capitalize_next = true;
				}
			}
			result
		}
	}
}

fn parse_and_extract_slice(s: &str, slice_str: &str) -> String {
	let chars: Vec<char> = s.chars().collect();
	if chars.is_empty() {
		return String::new();
	}

	let get_index = |num_str: &str, default: usize| -> usize {
		if num_str.is_empty() {
			return default;
		}
		if let Ok(val) = num_str.parse::<isize>() {
			if val < 0 {
				let abs_val = val.unsigned_abs();
				chars.len().saturating_sub(abs_val)
			} else if val > 0 {
				(val as usize).saturating_sub(1).min(chars.len())
			} else {
				0
			}
		} else {
			default
		}
	};

	if slice_str.is_empty() {
		return s.to_string();
	}

	if slice_str.contains(',') {
		let parts: Vec<&str> = slice_str.split(',').collect();
		let start_idx = get_index(parts[0], 0);
		let len = parts.get(1).and_then(|p| p.parse::<usize>().ok()).unwrap_or(chars.len());
		let end_idx = (start_idx + len).min(chars.len());
		if start_idx <= end_idx {
			return chars[start_idx..end_idx].iter().collect();
		}
		return String::new();
	}

	if slice_str.contains('-') {
		let mut split_pos = None;
		let bytes = slice_str.as_bytes();
		for i in 1..bytes.len() {
			if bytes[i] == b'-' {
				let start_from = if bytes[0] == b'-' { 1 } else { 0 };
				if let Some(pos) = slice_str[start_from..].find('-') {
					split_pos = Some(start_from + pos);
					break;
				}
			}
		}

		if let Some(pos) = split_pos {
			let part1 = &slice_str[..pos];
			let part2 = &slice_str[pos + 1..];
			let start_idx = get_index(part1, 0);
			let end_idx = if part2.is_empty() {
				chars.len()
			} else if part2.starts_with('-') {
				get_index(part2, chars.len())
			} else if let Ok(val) = part2.parse::<usize>() {
				val.min(chars.len())
			} else {
				chars.len()
			};

			let start_idx = start_idx.min(chars.len());
			let end_idx = end_idx.min(chars.len());
			if start_idx <= end_idx {
				return chars[start_idx..end_idx].iter().collect();
			}
			return String::new();
		} else {
			let start_idx = get_index(slice_str, 0);
			if start_idx < chars.len() {
				return chars[start_idx..].iter().collect();
			}
			return String::new();
		}
	}

	if let Ok(idx) = slice_str.parse::<usize>() {
		let char_idx = idx.saturating_sub(1);
		return chars.get(char_idx).map(|c| c.to_string()).unwrap_or_default();
	}

	s.to_string()
}

fn perform_replace(
	s: &str,
	find: &str,
	replace: &str,
	use_regex: bool,
	case_sensitive: bool,
	replace_1x: bool,
) -> String {
	if find.is_empty() {
		return s.to_string();
	}

	let find_parts: Vec<&str> = find.split('|').collect();
	let replace_parts: Vec<&str> = replace.split('|').collect();

	let mut result = s.to_string();

	for (i, &f) in find_parts.iter().enumerate() {
		if f.is_empty() {
			continue;
		}
		let r = replace_parts.get(i).copied().unwrap_or("");

		if use_regex {
			let pattern = if case_sensitive { f.to_string() } else { format!("(?i){}", f) };
			if let Ok(re) = Regex::new(&pattern) {
				if replace_1x {
					result = re.replace(&result, r).to_string();
				} else {
					result = re.replace_all(&result, r).to_string();
				}
			}
		} else {
			if case_sensitive {
				if replace_1x {
					if let Some(pos) = result.find(f) {
						result.replace_range(pos..pos + f.len(), r);
					}
				} else {
					result = result.replace(f, r);
				}
			} else {
				let escaped = regex::escape(f);
				let pattern = format!("(?i){}", escaped);
				if let Ok(re) = Regex::new(&pattern) {
					if replace_1x {
						result = re.replace(&result, r).to_string();
					} else {
						result = re.replace_all(&result, r).to_string();
					}
				}
			}
		}
	}

	result
}

fn generate_previews(
	files: &[String],
	mask: &str,
	ext_mask: &str,
	find: &str,
	replace: &str,
	use_regex: bool,
	case_sensitive: bool,
	replace_1x: bool,
	name_case: CaseConv,
	ext_case: CaseConv,
	c_start_str: &str,
	c_step_str: &str,
	c_width_str: &str,
) -> Vec<(String, String)> {
	let c_start = c_start_str.parse::<usize>().unwrap_or(1);
	let c_step = c_step_str.parse::<usize>().unwrap_or(1);
	let c_width = c_width_str.parse::<usize>().unwrap_or(3);

	let mut previews = Vec::new();

	for (file_idx, f) in files.iter().enumerate() {
		let old_name = Path::new(f).file_name().unwrap_or_default().to_string_lossy().to_string();

		let ext =
			Path::new(&old_name).extension().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();

		let name_without_ext = if !ext.is_empty() {
			old_name[..old_name.len() - ext.len() - 1].to_string()
		} else {
			old_name.clone()
		};

		let parent_path = Path::new(f).parent();
		let parent_dir = parent_path
			.and_then(|p| p.file_name())
			.map(|s| s.to_string_lossy().to_string())
			.unwrap_or_default();
		let grandparent_dir = parent_path
			.and_then(|p| p.parent())
			.and_then(|p| p.file_name())
			.map(|s| s.to_string_lossy().to_string())
			.unwrap_or_default();

		let mut new_name = parse_mask(
			mask,
			&name_without_ext,
			&ext,
			&parent_dir,
			&grandparent_dir,
			file_idx,
			c_start,
			c_step,
			c_width,
		);
		let mut new_ext = parse_mask(
			ext_mask,
			&name_without_ext,
			&ext,
			&parent_dir,
			&grandparent_dir,
			file_idx,
			c_start,
			c_step,
			c_width,
		);

		new_name = apply_case_conv(&new_name, name_case);
		new_ext = apply_case_conv(&new_ext, ext_case);

		let mut final_name =
			if !new_ext.is_empty() { format!("{}.{}", new_name, new_ext) } else { new_name };

		final_name = perform_replace(&final_name, find, replace, use_regex, case_sensitive, replace_1x);

		previews.push((old_name, final_name));
	}

	previews
}
