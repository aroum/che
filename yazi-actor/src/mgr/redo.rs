use anyhow::Result;
use yazi_fs::{File, FilesOp};
use yazi_macro::succ;
use yazi_parser::mgr::RedoOpt;
use yazi_proxy::MgrProxy;
use yazi_shared::{UndoOp, data::Data, url::UrlLike};
use yazi_vfs::{VfsFile, provider};
use yazi_watcher::WATCHER;

use crate::{Actor, Ctx};

pub struct Redo;

impl Actor for Redo {
	type Options = RedoOpt;

	const NAME: &str = "redo";

	fn act(cx: &mut Ctx, _opt: Self::Options) -> Result<Data> {
		let Some(entry) = cx.mgr.undo.redo() else { succ!() };

		if let UndoOp::Copy { ref pairs, .. } = entry.op {
			// Clear pairs/overwritten on the undo stack entry; hooks will repopulate
			if let Some(top) = cx.mgr.undo.undo_stack_last_mut() {
				if let UndoOp::Copy { pairs: ref mut p, overwritten: ref mut o } = top.op {
					p.clear();
					o.clear();
				}
			}
			for (from, to) in pairs {
				cx.core.tasks.scheduler.file_copy(from.clone(), to.clone(), true, false);
			}
			succ!();
		}

		if let UndoOp::Move { ref pairs, .. } = entry.op {
			// Clear pairs/overwritten on the undo stack entry; hooks will repopulate
			if let Some(top) = cx.mgr.undo.undo_stack_last_mut() {
				if let UndoOp::Move { pairs: ref mut p, overwritten: ref mut o } = top.op {
					p.clear();
					o.clear();
				}
			}
			for (from, to) in pairs {
				cx.core.tasks.scheduler.file_cut(from.clone(), to.clone(), true);
			}
			succ!();
		}

		tokio::spawn(async move {
			let _permit = WATCHER.acquire().await.unwrap();
			match entry.op {
				UndoOp::Rename { ref old, ref new } => {
					Self::redo_rename(old, new).await.ok();
				}
				UndoOp::Create { ref target, is_dir } => {
					Self::redo_create(target, is_dir).await.ok();
				}
				UndoOp::Copy { .. } | UndoOp::Move { .. } => unreachable!(),
				UndoOp::Trash { ref pairs } => {
					Self::redo_trash(pairs).await.ok();
				}
			}
		});
		succ!();
	}
}

impl Redo {
	async fn redo_rename(
		old: &yazi_shared::url::UrlBuf,
		new: &yazi_shared::url::UrlBuf,
	) -> Result<()> {
		provider::rename(old, new).await?;

		let Some((old_p, old_n)) = old.pair() else { return Ok(()) };
		let Some((new_p, new_n)) = new.pair() else { return Ok(()) };

		let file = File::new(new).await?;
		if old_p == new_p {
			FilesOp::Upserting(new_p.into(), [(old_n.into(), file)].into()).emit();
		} else {
			FilesOp::Deleting(old_p.into(), [old_n.into()].into()).emit();
			FilesOp::Upserting(new_p.into(), [(new_n.into(), file)].into()).emit();
		}
		Ok(())
	}

	async fn redo_create(target: &yazi_shared::url::UrlBuf, is_dir: bool) -> Result<()> {
		if is_dir {
			provider::create_dir_all(target).await?;
		} else {
			if let Some(parent) = target.parent() {
				provider::create_dir_all(parent).await.ok();
			}
			provider::create(target).await?;
		}

		if let Some((parent, urn)) = target.pair() {
			if let Ok(file) = File::new(target).await {
				FilesOp::Upserting(parent.into(), [(urn.into(), file)].into()).emit();
			}
		}
		Ok(())
	}

	async fn redo_trash(
		pairs: &[(yazi_shared::url::UrlBuf, yazi_shared::url::UrlBuf)],
	) -> Result<()> {
		let mut new_pairs = Vec::new();
		for (original, _) in pairs {
			if let Ok(new_trash_path) = provider::trash(original).await {
				if let Some((orig_p, orig_n)) = original.pair() {
					FilesOp::Deleting(orig_p.into(), [orig_n.into()].into()).emit();
				}
				new_pairs.push((original.clone(), new_trash_path));
			}
		}
		if !new_pairs.is_empty() {
			MgrProxy::undo_push(UndoOp::Trash { pairs: new_pairs });
		}
		Ok(())
	}
}
