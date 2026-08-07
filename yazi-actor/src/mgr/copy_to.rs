use anyhow::Result;
use yazi_config::popup::ConfirmCfg;
use yazi_core::mgr::Yanked;
use yazi_macro::{act, emit, relay, render, succ};
use yazi_parser::mgr::CopyToOpt;
use yazi_proxy::{ConfirmProxy, NotifyProxy};
use yazi_shared::{UndoOp, data::Data, url::{UrlBufCov, UrlLike}};
use yazi_vfs::maybe_exists;

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

		if opt.force {
			cx.core.tasks.file_copy(&yanked, &dest, true, false);
			cx.mgr.undo.push(UndoOp::Copy { pairs: vec![], overwritten: vec![] });
			act!(mgr:escape_select, cx)?;
			succ!();
		}

		tokio::spawn(async move {
			let mut has_conflict = false;
			for u in yanked.iter() {
				if let Some(Ok(to)) = u.name().map(|n| dest.try_join(n)) {
					if maybe_exists(&to).await {
						has_conflict = true;
						if !ConfirmProxy::show(ConfirmCfg::overwrite(&to)).await {
							return;
						}
						break;
					}
				}
			}

			emit!(Call(relay!(mgr:copy_to).with("force", has_conflict)));
		});

		succ!()
	}
}
