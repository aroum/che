use anyhow::Result;
use yazi_macro::{act, render, succ};
use yazi_parser::mgr::TabRestoreOpt;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct TabRestore;

impl Actor for TabRestore {
	type Options = TabRestoreOpt;

	const NAME: &str = "tab_restore";

	fn act(cx: &mut Ctx, _: Self::Options) -> Result<Data> {
		let pane = cx.tabs_mut().active_pane_mut();
		if let Some(tab) = pane.pop_closed() {
			let idx = usize::min(pane.cursor + 1, pane.items.len());
			pane.items.insert(idx, tab);
			pane.set_idx(idx);

			let cx = &mut Ctx::renew(cx);
			act!(mgr:refresh, cx)?;
			act!(mgr:peek, cx, true)?;
			act!(app:title, cx).ok();

			succ!(render!());
		}

		succ!();
	}
}
