use std::{fmt::{Display, Write}, str::FromStr};

use anyhow::bail;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::map_cyrillic;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Key {
	pub code:   KeyCode,
	pub shift:  bool,
	pub ctrl:   bool,
	pub alt:    bool,
	pub super_: bool,
}

impl Key {
	#[inline]
	pub fn plain(&self) -> Option<char> {
		match self.code {
			KeyCode::Char(c) if !self.ctrl && !self.alt && !self.super_ => Some(c),
			_ => None,
		}
	}
}

impl Default for Key {
	fn default() -> Self {
		Self { code: KeyCode::Null, shift: false, ctrl: false, alt: false, super_: false }
	}
}

impl From<KeyEvent> for Key {
	fn from(mut value: KeyEvent) -> Self {
		if let KeyCode::Char(c) = value.code {
			value.code = KeyCode::Char(map_cyrillic(c));
		}
		// For alphabet:
		//   Unix    :  <S-a> => Char("A") + SHIFT
		//   Windows :  <S-a> => Char("A") + SHIFT
		//
		// For non-alphabet:
		//   Unix    :  <S-`> => Char("~") + NULL
		//   Windows :  <S-`> => Char("~") + SHIFT
		//
		// So we detect `Char("~") + SHIFT`, and change it to `Char("~") + NULL`
		// for consistent behavior between OSs.

		let shift = match (value.code, value.modifiers) {
			(KeyCode::Char(c), m) => c.is_ascii_uppercase() || m.contains(KeyModifiers::SHIFT),
			(KeyCode::BackTab, _) => false,
			(_, m) => m.contains(KeyModifiers::SHIFT),
		};

		if shift && let KeyCode::Char(c) = value.code && c.is_ascii_lowercase() {
			value.code = KeyCode::Char(c.to_ascii_uppercase());
		}

		Self {
			code: value.code,
			shift,
			ctrl: value.modifiers.contains(KeyModifiers::CONTROL),
			alt: value.modifiers.contains(KeyModifiers::ALT),
			super_: value.modifiers.contains(KeyModifiers::SUPER),
		}
	}
}

impl FromStr for Key {
	type Err = anyhow::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		if s.is_empty() {
			bail!("empty key")
		}

		let mut key = Self::default();
		if !s.starts_with('<') || !s.ends_with('>') {
			key.code = KeyCode::Char(s.chars().next().unwrap());
			key.shift = matches!(key.code, KeyCode::Char(c) if c.is_ascii_uppercase());
			return Ok(key);
		}

		let mut it = s[1..s.len() - 1].split_inclusive('-').peekable();
		while let Some(next) = it.next() {
			match next.to_ascii_lowercase().as_str() {
				"s-" => key.shift = true,
				"c-" => key.ctrl = true,
				"a-" => key.alt = true,
				"d-" => key.super_ = true,

				"space" => key.code = KeyCode::Char(' '),
				"backspace" => key.code = KeyCode::Backspace,
				"enter" => key.code = KeyCode::Enter,
				"left" => key.code = KeyCode::Left,
				"right" => key.code = KeyCode::Right,
				"up" => key.code = KeyCode::Up,
				"down" => key.code = KeyCode::Down,
				"home" => key.code = KeyCode::Home,
				"end" => key.code = KeyCode::End,
				"pageup" => key.code = KeyCode::PageUp,
				"pagedown" => key.code = KeyCode::PageDown,
				"tab" => key.code = KeyCode::Tab,
				"backtab" => key.code = KeyCode::BackTab,
				"delete" => key.code = KeyCode::Delete,
				"insert" => key.code = KeyCode::Insert,
				"f1" => key.code = KeyCode::F(1),
				"f2" => key.code = KeyCode::F(2),
				"f3" => key.code = KeyCode::F(3),
				"f4" => key.code = KeyCode::F(4),
				"f5" => key.code = KeyCode::F(5),
				"f6" => key.code = KeyCode::F(6),
				"f7" => key.code = KeyCode::F(7),
				"f8" => key.code = KeyCode::F(8),
				"f9" => key.code = KeyCode::F(9),
				"f10" => key.code = KeyCode::F(10),
				"f11" => key.code = KeyCode::F(11),
				"f12" => key.code = KeyCode::F(12),
				"f13" => key.code = KeyCode::F(13),
				"f14" => key.code = KeyCode::F(14),
				"f15" => key.code = KeyCode::F(15),
				"f16" => key.code = KeyCode::F(16),
				"f17" => key.code = KeyCode::F(17),
				"f18" => key.code = KeyCode::F(18),
				"f19" => key.code = KeyCode::F(19),
				"esc" => key.code = KeyCode::Esc,

				_ => match next {
					s if it.peek().is_none() => {
						let c = s.chars().next().unwrap();
						key.shift |= c.is_ascii_uppercase();
						key.code = KeyCode::Char(if key.shift { c.to_ascii_uppercase() } else { c });
					}
					s => bail!("unknown key: {s}"),
				},
			}
		}

		if key.code == KeyCode::Null {
			bail!("empty key")
		}
		Ok(key)
	}
}

impl Display for Key {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		if let Some(c) = self.plain() {
			return if c == ' ' { write!(f, "<Space>") } else { f.write_char(c) };
		}

		write!(f, "<")?;
		if self.super_ {
			write!(f, "D-")?;
		}
		if self.ctrl {
			write!(f, "C-")?;
		}
		if self.alt {
			write!(f, "A-")?;
		}
		if self.shift && !matches!(self.code, KeyCode::Char(_)) {
			write!(f, "S-")?;
		}

		let code = match self.code {
			KeyCode::Backspace => "Backspace",
			KeyCode::Enter => "Enter",
			KeyCode::Left => "Left",
			KeyCode::Right => "Right",
			KeyCode::Up => "Up",
			KeyCode::Down => "Down",
			KeyCode::Home => "Home",
			KeyCode::End => "End",
			KeyCode::PageUp => "PageUp",
			KeyCode::PageDown => "PageDown",
			KeyCode::Tab => "Tab",
			KeyCode::BackTab => "BackTab",
			KeyCode::Delete => "Delete",
			KeyCode::Insert => "Insert",
			KeyCode::F(1) => "F1",
			KeyCode::F(2) => "F2",
			KeyCode::F(3) => "F3",
			KeyCode::F(4) => "F4",
			KeyCode::F(5) => "F5",
			KeyCode::F(6) => "F6",
			KeyCode::F(7) => "F7",
			KeyCode::F(8) => "F8",
			KeyCode::F(9) => "F9",
			KeyCode::F(10) => "F10",
			KeyCode::F(11) => "F11",
			KeyCode::F(12) => "F12",
			KeyCode::F(13) => "F13",
			KeyCode::F(14) => "F14",
			KeyCode::F(15) => "F15",
			KeyCode::F(16) => "F16",
			KeyCode::F(17) => "F17",
			KeyCode::F(18) => "F18",
			KeyCode::F(19) => "F19",
			KeyCode::Esc => "Esc",

			KeyCode::Char(' ') => "Space",
			KeyCode::Char(c) => {
				f.write_char(c)?;
				""
			}
			_ => "Unknown",
		};

		if code.is_empty() {
			write!(f, ">")
		} else {
			write!(f, "{code}>")
		}
	}
}

#[cfg(test)]
mod tests {
	use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

	use super::*;

	#[test]
	fn test_cyrillic_mapping_and_modifiers() {
		// Russian 'к' maps to 'r'
		let k_event = KeyEvent::new(KeyCode::Char('к'), KeyModifiers::NONE);
		let key_k = Key::from(k_event);
		assert_eq!(key_k.code, KeyCode::Char('r'));
		assert!(!key_k.shift);

		// Russian 'К' (Shift + 'к') maps to 'R' with shift flag
		let shift_k_event = KeyEvent::new(KeyCode::Char('К'), KeyModifiers::SHIFT);
		let key_shift_k = Key::from(shift_k_event);
		assert_eq!(key_shift_k.code, KeyCode::Char('R'));
		assert!(key_shift_k.shift);

		// Ctrl + Shift + Russian 'к' maps to Ctrl + Shift + 'R' (<C-S-r>)
		let mods = KeyModifiers::CONTROL | KeyModifiers::SHIFT;
		let ctrl_shift_k = KeyEvent::new(KeyCode::Char('к'), mods);
		let key_ctrl_shift_k = Key::from(ctrl_shift_k);
		assert_eq!(key_ctrl_shift_k.code, KeyCode::Char('R'));
		assert!(key_ctrl_shift_k.ctrl);
		assert!(key_ctrl_shift_k.shift);

		// Russian 'о' maps to 'j'
		let o_event = KeyEvent::new(KeyCode::Char('о'), KeyModifiers::NONE);
		assert_eq!(Key::from(o_event).code, KeyCode::Char('j'));
	}
}
