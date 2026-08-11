use anyhow::Result;
use yazi_config::popup::ConfirmCfg;
use yazi_core::mgr::Yanked;
use yazi_macro::{act, emit, relay, render, succ};
use yazi_parser::mgr::MoveToOpt;
use yazi_proxy::{ConfirmProxy, NotifyProxy};
use yazi_shared::{UndoOp, data::Data, url::{UrlBufCov, UrlLike}};
use yazi_vfs::maybe_exists;

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

		if opt.force {
			cx.core.tasks.file_cut(&yanked, &dest, true);
			cx.mgr.undo.push(UndoOp::Move { pairs: vec![], overwritten: vec![] });
			act!(mgr:escape_select, cx)?;
			act!(mgr:unyank, cx)?;
			succ!();
		}

		tokio::spawn(async move {
			for u in yanked.iter() {
				if let Some(Ok(to)) = u.name().map(|n| dest.try_join(n)) {
					if maybe_exists(&to).await {
						if !ConfirmProxy::show(ConfirmCfg::overwrite(&to)).await {
							return;
						}
						break;
					}
				}
			}

			emit!(Call(relay!(mgr:move_to).with("force", true)));
		});

		succ!()
	}
}
