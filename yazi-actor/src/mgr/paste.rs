use anyhow::Result;
use yazi_macro::{act, succ};
use yazi_parser::mgr::PasteOpt;
use yazi_shared::{UndoOp, data::Data, url::UrlLike};

use crate::{Actor, Ctx};

pub struct Paste;

impl Actor for Paste {
	type Options = PasteOpt;

	const NAME: &str = "paste";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		let mgr = &mut cx.core.mgr;
		let tab = &mgr.tabs.panes[cx.pane].items[cx.tab];

		let dest = tab.cwd().clone();
		if mgr.yanked.cut {
			if mgr.yanked.iter().any(|u| u.parent().map_or(false, |p| p == dest)) {
				yazi_proxy::NotifyProxy::push_warn("Paste", "Source and destination directories are the same");
				succ!();
			}

			cx.core.tasks.file_cut(&mgr.yanked, &dest, opt.force);
			mgr.undo.push(UndoOp::Move { pairs: vec![], overwritten: vec![] });

			mgr.tabs.all_tabs_mut().for_each(|t| _ = t.selected.remove_many(mgr.yanked.iter()));
			act!(mgr:unyank, cx)
		} else {
			cx.core.tasks.file_copy(&mgr.yanked, &dest, opt.force, opt.follow);
			mgr.undo.push(UndoOp::Copy { pairs: vec![], overwritten: vec![] });
			succ!();
		}
	}
}
