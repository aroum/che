use anyhow::Result;
use yazi_actor::Ctx;
use yazi_config::{KEYMAP, keymap::{Chord, ChordCow, Key}};
use yazi_macro::{act, emit};
use yazi_shared::Layer;

use crate::app::App;

pub(super) struct Router<'a> {
	app: &'a mut App,
}

impl<'a> Router<'a> {
	pub(super) fn new(app: &'a mut App) -> Self { Self { app } }

	pub(super) fn route(&mut self, mut key: Key) -> Result<bool> {
		let core = &mut self.app.core;
		let layer = core.layer();

		if !core.input.visible
			&& let crossterm::event::KeyCode::Char(c) = key.code
				&& let Some(qwerty_c) = Self::cyrillic_to_qwerty(c) {
					key.code = crossterm::event::KeyCode::Char(qwerty_c);
					key.shift = qwerty_c.is_ascii_uppercase();
				}

		if core.help.visible && core.help.r#type(&key)? {
			return Ok(true);
		}
		if core.input.visible && core.input.r#type(&key)? {
			return Ok(true);
		}

		if core.pick.visible && !key.ctrl && !key.alt
			&& let crossterm::event::KeyCode::Char(c) = key.code {
				use yazi_core::pick::PickJumpResult as R;
				match core.pick.jump_letter(c) {
					R::Submit => {
						let cx = &mut Ctx::active(&mut self.app.core, &mut self.app.term);
						act!(pick:close, cx, true).ok();
						return Ok(true);
					}
					R::Moved => {
						yazi_macro::render!();
						return Ok(true);
					}
					R::None => {}
				}
			}

		if layer == Layer::Mgr && !core.input.visible && !core.confirm.visible && !core.help.visible {
			let is_jump_mode = core.active().jump_mode;

			if key.ctrl && !key.alt && matches!(key.code, crossterm::event::KeyCode::Char('j' | 'J')) {
				let cx = &mut Ctx::active(&mut self.app.core, &mut self.app.term);
				act!(mgr:jump_mode, cx, ()).ok();
				return Ok(true);
			}

			if is_jump_mode {
				if matches!(key.code, crossterm::event::KeyCode::Esc) {
					let cx = &mut Ctx::active(&mut self.app.core, &mut self.app.term);
					act!(mgr:jump_mode, cx, false).ok();
					return Ok(true);
				}

				if !key.ctrl && !key.alt
					&& let crossterm::event::KeyCode::Char(c) = key.code {
						let cx = &mut Ctx::active(&mut self.app.core, &mut self.app.term);
						act!(mgr:jump_letter, cx, c).ok();
						return Ok(true);
					}
			}
		}

		use Layer as L;
		Ok(match layer {
			L::App | L::Notify => unreachable!(),
			L::Mgr | L::Tasks | L::Spot | L::Pick | L::Input | L::Confirm | L::Help => {
				self.matches(layer, key)
			}
			L::Cmp => self.matches(L::Cmp, key) || self.matches(L::Input, key),
			L::Which => core.which.r#type(key),
		})
	}

	fn cyrillic_to_qwerty(c: char) -> Option<char> {
		match c {
			// Lowercase
			'й' => Some('q'),
			'ц' => Some('w'),
			'у' => Some('e'),
			'к' => Some('r'),
			'е' => Some('t'),
			'н' => Some('y'),
			'г' => Some('u'),
			'ш' => Some('i'),
			'щ' => Some('o'),
			'з' => Some('p'),
			'х' => Some('['),
			'ъ' => Some(']'),
			'ф' => Some('a'),
			'ы' => Some('s'),
			'в' => Some('d'),
			'а' => Some('f'),
			'п' => Some('g'),
			'р' => Some('h'),
			'о' => Some('j'),
			'л' => Some('k'),
			'д' => Some('l'),
			'ж' => Some(';'),
			'э' => Some('\''),
			'я' => Some('z'),
			'ч' => Some('x'),
			'с' => Some('c'),
			'м' => Some('v'),
			'и' => Some('b'),
			'т' => Some('n'),
			'ь' => Some('m'),
			'б' => Some(','),
			'ю' => Some('.'),
			'ё' => Some('`'),

			// Uppercase
			'Й' => Some('Q'),
			'Ц' => Some('W'),
			'У' => Some('E'),
			'К' => Some('R'),
			'Е' => Some('T'),
			'Н' => Some('Y'),
			'Г' => Some('U'),
			'Ш' => Some('I'),
			'Щ' => Some('O'),
			'З' => Some('P'),
			'Х' => Some('{'),
			'Ъ' => Some('}'),
			'Ф' => Some('A'),
			'Ы' => Some('S'),
			'В' => Some('D'),
			'А' => Some('F'),
			'П' => Some('G'),
			'Р' => Some('H'),
			'О' => Some('J'),
			'Л' => Some('K'),
			'Д' => Some('L'),
			'Ж' => Some(':'),
			'Э' => Some('"'),
			'Я' => Some('Z'),
			'Ч' => Some('X'),
			'С' => Some('C'),
			'М' => Some('V'),
			'И' => Some('B'),
			'Т' => Some('N'),
			'Ь' => Some('M'),
			'Б' => Some('<'),
			'Ю' => Some('>'),
			'Ё' => Some('~'),

			_ => None,
		}
	}

	fn matches(&mut self, layer: Layer, key: Key) -> bool {
		for chord @ Chord { on, .. } in KEYMAP.get(layer) {
			if on.is_empty() || on[0] != key {
				continue;
			}

			if on.len() > 1 {
				let cx = &mut Ctx::active(&mut self.app.core, &mut self.app.term);
				act!(which:activate, cx, (layer, key)).ok();
			} else {
				emit!(Seq(ChordCow::from(chord).into_seq()));
			}
			return true;
		}
		false
	}
}
