use anyhow::Result;
use yazi_core::mgr::Yanked;
use yazi_macro::{act, render, succ};
use yazi_parser::mgr::CopyToOpt;
use yazi_proxy::NotifyProxy;
use yazi_shared::{UndoOp, data::Data, url::UrlBufCov};

use crate::{Actor, Ctx};

pub struct CopyTo;

impl Actor for CopyTo {
	type Options = CopyToOpt;

	const NAME: &str = "copy_to";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		act!(mgr:escape_visual, cx)?;

		let yanked =
			Yanked::new(false, cx.tab().selected_or_hovered().cloned().map(UrlBufCov).collect());
		if yanked.is_empty() {
			succ!();
		}

		let dest = cx.tabs().other().cwd().clone();

		if cx.tab().cwd() == &dest {
			NotifyProxy::push_warn("Copy", "Both panes are in the same directory");
			succ!(render!());
		}

		cx.core.tasks.file_copy(&yanked, &dest, opt.force, false);
		cx.mgr.undo.push(UndoOp::Copy { pairs: vec![], overwritten: vec![] });
		act!(mgr:escape_select, cx)
	}
}
