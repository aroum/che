use anyhow::Result;
use yazi_macro::{act, render, succ};
use yazi_parser::VoidOpt;
use yazi_shared::data::Data;
use yazi_widgets::input::InputOp;

use crate::{Actor, Ctx};

pub struct Escape;

impl Actor for Escape {
	type Options = VoidOpt;

	const NAME: &str = "escape";

	fn act(cx: &mut Ctx, _: Self::Options) -> Result<Data> {
		use yazi_widgets::input::InputMode as M;
		let input = &mut cx.input;

		use yazi_config::YAZI;
		let mode = input.snap().mode;
		if YAZI.input.vim_mode {
			match mode {
				M::Normal if input.snap_mut().op == InputOp::None => act!(input:close, cx),
				M::Insert if cx.cmp.visible => act!(cmp:close, cx),
				M::Insert | M::Normal | M::Replace => Ok(().into()),
			}?;
		} else {
			match mode {
				M::Insert if cx.cmp.visible => act!(cmp:close, cx),
				_ => act!(input:close, cx),
			}?;
		}

		act!(escape, cx.input)?;
		succ!(render!());
	}
}
