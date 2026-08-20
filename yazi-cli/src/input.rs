use ratatui::{
	style::{Color, Modifier, Style},
	text::Span,
};

#[derive(Clone, Debug, Default)]
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

	#[inline]
	pub fn left(&mut self) {
		self.move_left();
	}

	#[inline]
	pub fn right(&mut self) {
		self.move_right();
	}

	#[inline]
	pub fn home(&mut self) {
		self.move_home();
	}

	#[inline]
	pub fn end(&mut self) {
		self.move_end();
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
