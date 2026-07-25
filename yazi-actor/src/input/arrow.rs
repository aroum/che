use anyhow::Result;
use yazi_macro::{act, render, succ};
use yazi_parser::ArrowOpt;
use yazi_shared::data::Data;
use yazi_widgets::input::InputError;

use crate::{Actor, Ctx};

pub struct Arrow;

impl Actor for Arrow {
	type Options = ArrowOpt;

	const NAME: &str = "arrow";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		let input = &mut cx.input;
		input.visible = false;
		input.ticket.next();

		if let Some(tx) = input.tx.take() {
			let value = input.snap().value.clone();
			let step = match opt.step {
				yazi_widgets::Step::Top => 0,
				yazi_widgets::Step::Bot => 0,
				yazi_widgets::Step::Prev => -1,
				yazi_widgets::Step::Next => 1,
				yazi_widgets::Step::Offset(n) => n,
				yazi_widgets::Step::Percent(_) => 0,
			};
			_ = tx.send(Err(InputError::Arrow(value, step)));
		}

		act!(cmp:close, cx)?;
		succ!(render!());
	}
}
