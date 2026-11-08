use anyhow::Result;
use yazi_fs::{File, FilesOp};
use yazi_macro::succ;
use yazi_parser::mgr::UndoOpt;
use yazi_shared::{UndoOp, data::Data, url::UrlLike};
use yazi_vfs::{VfsFile, provider};
use yazi_watcher::WATCHER;

use crate::{Actor, Ctx};

pub struct Undo;

impl Actor for Undo {
	type Options = UndoOpt;

	const NAME: &str = "undo";

	fn act(cx: &mut Ctx, _opt: Self::Options) -> Result<Data> {
		let Some(entry) = cx.mgr.undo.undo() else { succ!() };

		tokio::spawn(async move {
			let _permit = WATCHER.acquire().await.unwrap();
			match entry.op {
				UndoOp::Rename { ref old, ref new } => {
					Self::undo_rename(old, new).await.ok();
				}
				UndoOp::Create { ref target, is_dir } => {
					Self::undo_create(target, is_dir).await.ok();
				}
				UndoOp::Copy { ref pairs, ref overwritten } => {
					Self::undo_copy(pairs, overwritten).await.ok();
				}
				UndoOp::Move { ref pairs, ref overwritten } => {
					Self::undo_move(pairs, overwritten).await.ok();
				}
				UndoOp::Trash { ref pairs } => {
					Self::undo_trash(pairs).await.ok();
				}
			}
		});
		succ!();
	}
}

impl Undo {
	async fn undo_rename(
		old: &yazi_shared::url::UrlBuf,
		new: &yazi_shared::url::UrlBuf,
	) -> Result<()> {
		provider::rename(new, old).await?;

		let Some((old_p, old_n)) = old.pair() else { return Ok(()) };
		let Some((new_p, new_n)) = new.pair() else { return Ok(()) };

		let file = File::new(old).await?;
		if old_p == new_p {
			FilesOp::Upserting(old_p.into(), [(old_n.into(), file)].into()).emit();
		} else {
			FilesOp::Deleting(new_p.into(), [new_n.into()].into()).emit();
			FilesOp::Upserting(old_p.into(), [(old_n.into(), file)].into()).emit();
		}
		Ok(())
	}

	async fn undo_create(target: &yazi_shared::url::UrlBuf, is_dir: bool) -> Result<()> {
		if is_dir {
			provider::remove_dir_all(target).await?;
		} else {
			provider::remove_file(target).await?;
		}

		if let Some((parent, urn)) = target.pair() {
			FilesOp::Deleting(parent.into(), [urn.into()].into()).emit();
		}
		Ok(())
	}

	async fn undo_copy(
		pairs: &[(yazi_shared::url::UrlBuf, yazi_shared::url::UrlBuf)],
		overwritten: &[(yazi_shared::url::UrlBuf, yazi_shared::url::UrlBuf)],
	) -> Result<()> {
		for (_, dest) in pairs {
			if provider::remove_file(dest).await.is_ok() || provider::remove_dir_all(dest).await.is_ok() {
				if let Some((parent, urn)) = dest.pair() {
					FilesOp::Deleting(parent.into(), [urn.into()].into()).emit();
				}
			}
		}
		for (original, trash_path) in overwritten {
			if provider::rename(trash_path, original).await.is_ok() {
				if let Some((trash_p, trash_n)) = trash_path.pair() {
					FilesOp::Deleting(trash_p.into(), [trash_n.into()].into()).emit();
				}
				if let Some((orig_p, orig_n)) = original.pair() {
					if let Ok(file) = File::new(original).await {
						FilesOp::Upserting(orig_p.into(), [(orig_n.into(), file)].into()).emit();
					}
				}
			}
		}
		Ok(())
	}

	async fn undo_move(
		pairs: &[(yazi_shared::url::UrlBuf, yazi_shared::url::UrlBuf)],
		overwritten: &[(yazi_shared::url::UrlBuf, yazi_shared::url::UrlBuf)],
	) -> Result<()> {
		for (from, to) in pairs {
			if provider::rename(to, from).await.is_ok() {
				if let Some((to_p, to_n)) = to.pair() {
					FilesOp::Deleting(to_p.into(), [to_n.into()].into()).emit();
				}
				if let Some((from_p, from_n)) = from.pair() {
					if let Ok(file) = File::new(from).await {
						FilesOp::Upserting(from_p.into(), [(from_n.into(), file)].into()).emit();
					}
				}
			}
		}
		for (original, trash_path) in overwritten {
			if provider::rename(trash_path, original).await.is_ok() {
				if let Some((trash_p, trash_n)) = trash_path.pair() {
					FilesOp::Deleting(trash_p.into(), [trash_n.into()].into()).emit();
				}
				if let Some((orig_p, orig_n)) = original.pair() {
					if let Ok(file) = File::new(original).await {
						FilesOp::Upserting(orig_p.into(), [(orig_n.into(), file)].into()).emit();
					}
				}
			}
		}
		Ok(())
	}

	async fn undo_trash(
		pairs: &[(yazi_shared::url::UrlBuf, yazi_shared::url::UrlBuf)],
	) -> Result<()> {
		for (original, trash_path) in pairs {
			if provider::rename(trash_path, original).await.is_ok() {
				if let Some((trash_p, trash_n)) = trash_path.pair() {
					FilesOp::Deleting(trash_p.into(), [trash_n.into()].into()).emit();
				}
				if let Some((orig_p, orig_n)) = original.pair() {
					if let Ok(file) = File::new(original).await {
						FilesOp::Upserting(orig_p.into(), [(orig_n.into(), file)].into()).emit();
					}
				}
			}
		}
		Ok(())
	}
}
