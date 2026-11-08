use std::iter;

use anyhow::Result;
use yazi_macro::succ;
use yazi_parser::VoidOpt;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Watch;

impl Actor for Watch {
	type Options = VoidOpt;

	const NAME: &str = "watch";

	fn act(cx: &mut Ctx, _: Self::Options) -> Result<Data> {
		let tabs = &cx.core.mgr.tabs;
		let it = tabs
			.all_tabs()
			.flat_map(|t| iter::once(t.cwd()).chain(t.parent.as_ref().map(|p| &p.url)))
			.chain(tabs.hovered().filter(|h| h.is_dir()).map(|h| &h.url));

		cx.core.mgr.watcher.watch(it);
		succ!();
	}
}
