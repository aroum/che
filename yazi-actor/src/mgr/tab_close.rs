use anyhow::Result;
use yazi_macro::{act, render, succ};
use yazi_parser::mgr::TabCloseOpt;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct TabClose;

impl Actor for TabClose {
	type Options = TabCloseOpt;

	const NAME: &str = "tab_close";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		let pane = cx.tabs_mut().active_pane_mut();
		let len = pane.len();
		if len < 2 || opt.idx >= len {
			succ!();
		}

		let tab = pane.items.remove(opt.idx);
		pane.push_closed(tab);

		if opt.idx > pane.cursor {
			pane.set_idx(pane.cursor);
		} else {
			pane.set_idx(usize::min(pane.cursor + 1, pane.len() - 1));
		}

		let cx = &mut Ctx::renew(cx);
		act!(mgr:refresh, cx)?;
		act!(mgr:peek, cx, true)?;
		act!(app:title, cx).ok();

		succ!(render!());
	}
}
