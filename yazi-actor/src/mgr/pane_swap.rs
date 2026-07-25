use anyhow::Result;
use yazi_macro::{act, render, succ};
use yazi_parser::mgr::{CdSource, PaneSwapOpt};
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct PaneSwap;

impl Actor for PaneSwap {
	type Options = PaneSwapOpt;

	const NAME: &str = "pane_swap";

	fn act(cx: &mut Ctx, _: Self::Options) -> Result<Data> {
		let current_pane = cx.tabs().active_pane;
		let other_pane = 1 - current_pane;

		let current_cwd = cx.tabs().active().current.url.clone();
		let other_cwd = cx.tabs().other().current.url.clone();

		// 1. Change path in the current pane
		act!(mgr:cd, cx, (other_cwd, CdSource::Cd))?;

		// 2. Switch to the other pane
		cx.tabs_mut().set_active_pane(other_pane);
		let cx = &mut Ctx::renew(cx);

		// 3. Change path in the other pane
		act!(mgr:cd, cx, (current_cwd, CdSource::Cd))?;

		// 4. Return focus to the original pane
		cx.tabs_mut().set_active_pane(current_pane);
		let _cx = &mut Ctx::renew(cx);

		succ!(render!())
	}
}
