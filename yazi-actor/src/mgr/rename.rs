use anyhow::Result;
use yazi_config::popup::{ConfirmCfg, InputCfg};
use yazi_dds::Pubsub;
use yazi_fs::{File, FilesOp};
use yazi_macro::{act, err, ok_or_not_found, succ};
use yazi_parser::mgr::RenameOpt;
use yazi_proxy::{ConfirmProxy, InputProxy, MgrProxy};
use yazi_shared::{Id, UndoOp, data::Data, url::{UrlBuf, UrlLike}};
use yazi_vfs::{VfsFile, maybe_exists, provider};
use yazi_watcher::WATCHER;

use yazi_widgets::input::InputError;
use crate::{Actor, Ctx};

pub struct Rename;

impl Actor for Rename {
	type Options = RenameOpt;

	const NAME: &str = "rename";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		tracing::info!("Rename action act() called with opt: {:?}", opt);
		act!(mgr:escape_visual, cx)?;

		if !opt.hovered && !cx.tab().selected.is_empty() {
			return act!(mgr:bulk_rename, cx);
		}

		let Some(hovered) = cx.hovered() else { succ!() };
		if hovered.is_upparent {
			succ!();
		}

		let name = Self::empty_url_part(&hovered.url, &opt.empty);
		let cursor = match opt.cursor.as_ref() {
			"start" => Some(0),
			"before_ext" => {
				let len = name.chars().count();
				name
					.chars()
					.rev()
					.position(|c| c == '.')
					.filter(|_| hovered.is_file())
					.map(|i| len - i - 1)
					.filter(|&i| i != 0)
					.or(Some(len))
			}
			_ => None,
		};

		let (tab, old) = (cx.tab().id, hovered.url_owned());
		let mut input = InputProxy::show(InputCfg::rename().with_value(name).with_cursor(cursor));

		let rename_opt = yazi_parser::mgr::RenameOpt {
			hovered: opt.hovered,
			force: opt.force,
			empty: opt.empty.clone(),
			cursor: opt.cursor.clone(),
		};

		tokio::spawn(async move {
			let result = input.recv().await;
			let (name, step) = match result {
				Some(Ok(name)) => (name, None),
				Some(Err(InputError::Arrow(name, step))) => (name, Some(step)),
				_ => return,
			};

			let old_name = old.name().map(|s| s.to_string_lossy()).unwrap_or_default();
			let changed = !name.is_empty() && name != old_name;

			if changed {
				if let Some(Ok(new)) = old.parent().map(|u| u.try_join(&name)) {
					if opt.force || !maybe_exists(&new).await || provider::must_identical(&old, &new).await {
						Self::r#do(tab, old, new).await.ok();
					} else if ConfirmProxy::show(ConfirmCfg::overwrite(&new)).await {
						Self::r#do(tab, old, new).await.ok();
					}
				}
			}

			if let Some(step) = step {
				MgrProxy::arrow(step.to_string());
				let action = yazi_macro::relay!(mgr:rename)
					.with("hovered", rename_opt.hovered)
					.with("force", rename_opt.force)
					.with("empty", rename_opt.empty)
					.with("cursor", rename_opt.cursor);
				yazi_macro::emit!(Call(action));
			}
		});
		succ!();
	}
}

impl Rename {
	async fn r#do(tab: Id, old: UrlBuf, new: UrlBuf) -> Result<()> {
		let Some((old_p, old_n)) = old.pair() else { return Ok(()) };
		let Some(_) = new.pair() else { return Ok(()) };
		let _permit = WATCHER.acquire().await.unwrap();

		let overwritten = provider::casefold(&new).await;
		provider::rename(&old, &new).await?;

		if let Ok(u) = overwritten
			&& u != new
			&& let Some((parent, urn)) = u.pair()
		{
			ok_or_not_found!(provider::rename(&u, &new).await);
			FilesOp::Deleting(parent.to_owned(), [urn.into()].into()).emit();
		}

		let new = provider::casefold(&new).await?;
		let Some((new_p, new_n)) = new.pair() else { return Ok(()) };

		let file = File::new(&new).await?;
		if new_p == old_p {
			FilesOp::Upserting(old_p.into(), [(old_n.into(), file)].into()).emit();
		} else {
			FilesOp::Deleting(old_p.into(), [old_n.into()].into()).emit();
			FilesOp::Upserting(new_p.into(), [(new_n.into(), file)].into()).emit();
		}

		MgrProxy::undo_push(UndoOp::Rename { old: old.clone(), new: new.clone() });
		MgrProxy::reveal(&new);
		err!(Pubsub::pub_after_rename(tab, &old, &new));
		Ok(())
	}

	fn empty_url_part(url: &UrlBuf, by: &str) -> String {
		if by == "all" {
			return String::new();
		}

		let ext = url.ext();
		match by {
			"stem" => ext.map_or_else(String::new, |s| format!(".{}", s.to_string_lossy())),
			"ext" if ext.is_some() => format!("{}.", url.stem().unwrap().to_string_lossy()),
			"dot_ext" if ext.is_some() => url.stem().unwrap().to_string_lossy().into_owned(),
			_ => url.name().unwrap_or_default().to_string_lossy().into_owned(),
		}
	}
}
