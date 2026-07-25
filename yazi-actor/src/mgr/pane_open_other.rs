use anyhow::Result;
use yazi_macro::{act, render, succ};
use yazi_parser::mgr::{CdSource, PaneOpenOtherOpt};
use yazi_shared::{data::Data, url::UrlLike};

use crate::{Actor, Ctx};

pub struct PaneOpenOther;

impl Actor for PaneOpenOther {
	type Options = PaneOpenOtherOpt;

	const NAME: &str = "pane_open_other";

	fn act(cx: &mut Ctx, _: Self::Options) -> Result<Data> {
		let current_pane = cx.tabs().active_pane;
		let other_pane = 1 - current_pane;

		let target_url = if let Some(hovered) = cx.hovered() {
			if hovered.is_upparent {
				cx.cwd().parent().map(|u| u.to_owned())
			} else if hovered.is_dir() {
				Some(hovered.url.clone())
			} else {
				Some(cx.cwd().clone())
			}
		} else {
			Some(cx.cwd().clone())
		};

		let Some(url) = target_url else { succ!() };

		// Switch to other pane
		cx.tabs_mut().set_active_pane(other_pane);
		let cx = &mut Ctx::renew(cx);

		// Navigate to target directory
		act!(mgr:cd, cx, (url, CdSource::Cd))?;

		// Switch back to original pane
		cx.tabs_mut().set_active_pane(current_pane);
		let _cx = &mut Ctx::renew(cx);

		succ!(render!())
	}
}
