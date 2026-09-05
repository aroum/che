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

	pub(super) fn route(&mut self, key: Key) -> Result<bool> {
		let core = &mut self.app.core;
		let layer = core.layer();

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

			if key.ctrl
				&& !key.alt
				&& matches!(key.code, crossterm::event::KeyCode::Char('j' | 'J' | 'о' | 'О'))
			{
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

		let key_qwerty = key.to_qwerty();

		use Layer as L;
		Ok(match layer {
			L::App | L::Notify => unreachable!(),
			L::Mgr | L::Tasks | L::Spot | L::Pick | L::Input | L::Confirm | L::Help => {
				self.matches(layer, key) || (key != key_qwerty && self.matches(layer, key_qwerty))
			}
			L::Cmp => {
				self.matches(L::Cmp, key)
					|| (key != key_qwerty && self.matches(L::Cmp, key_qwerty))
					|| self.matches(L::Input, key)
					|| (key != key_qwerty && self.matches(L::Input, key_qwerty))
			}
			L::Which => core.which.r#type(key) || (key != key_qwerty && core.which.r#type(key_qwerty)),
		})
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
