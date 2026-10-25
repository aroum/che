use anyhow::Result;
use yazi_macro::{act, render, succ};
use yazi_parser::mgr::VisualArrowOpt;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct VisualArrow;

impl Actor for VisualArrow {
	type Options = VisualArrowOpt;

	const NAME: &str = "visual_arrow";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		let tab = cx.tab_mut();
		let len = tab.current.files.len() as isize;
		if len == 0 {
			succ!();
		}

		let Some((start, items)) = tab.mode.visual_mut() else { succ!() };
		let new = tab.current.cursor as isize + opt.step;
		let new_cursor = new.rem_euclid(len) as usize;

		*start = tab.current.cursor;
		let end = new_cursor;
		*items = ((*start).min(end)..=end.max(*start)).collect();
		tab.current.cursor = new_cursor;

		tab.current.arrow(0);

		act!(mgr:hover, cx)?;
		act!(mgr:peek, cx)?;
		act!(mgr:watch, cx).ok();

		succ!(render!());
	}
}
