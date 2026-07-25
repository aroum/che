use anyhow::Result;
use yazi_core::mgr::Yanked;
use yazi_macro::{act, render, succ};
use yazi_parser::mgr::MoveToOpt;
use yazi_proxy::NotifyProxy;
use yazi_shared::{UndoOp, data::Data, url::UrlBufCov};

use crate::{Actor, Ctx};

pub struct MoveTo;

impl Actor for MoveTo {
	type Options = MoveToOpt;

	const NAME: &str = "move_to";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		act!(mgr:escape_visual, cx)?;

		let yanked =
			Yanked::new(true, cx.tab().selected_or_hovered().cloned().map(UrlBufCov).collect());
		if yanked.is_empty() {
			succ!();
		}

		let dest = cx.tabs().other().cwd().clone();

		if cx.tab().cwd() == &dest {
			NotifyProxy::push_warn("Move", "Both panes are in the same directory");
			succ!(render!());
		}

		cx.core.tasks.file_cut(&yanked, &dest, opt.force);
		cx.mgr.undo.push(UndoOp::Move { pairs: vec![], overwritten: vec![] });
		act!(mgr:escape_select, cx)?;
		act!(mgr:unyank, cx)
	}
}
