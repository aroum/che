use anyhow::Result;
use yazi_config::popup::InputCfg;
use yazi_macro::{act, succ};
use yazi_parser::{VoidOpt, mgr::CdSource};
use yazi_proxy::{InputProxy, MgrProxy, NotifyProxy};
use yazi_shared::{data::Data, url::UrlLike};

use crate::{Actor, Ctx};

pub struct Enter;

impl Actor for Enter {
	type Options = VoidOpt;

	const NAME: &str = "enter";

	fn act(cx: &mut Ctx, _: Self::Options) -> Result<Data> {
		let Some(h) = cx.hovered() else { succ!() };

		if h.is_dir() {
			let url = if h.url.is_search() { h.url.to_regular()? } else { h.url.clone() };
			act!(mgr:cd, cx, (url, CdSource::Enter))
		} else if yazi_config::YAZI.mgr.archive_vfs.get() && Self::is_archive_url(&h.url) {
			Self::enter_archive(&h.url)?;
			succ!()
		} else {
			succ!()
		}
	}
}

impl Enter {
	fn is_archive_url(url: &yazi_shared::url::UrlBuf) -> bool {
		url.ext().is_some_and(|ext| {
			let ext = ext.to_string_lossy().to_lowercase();
			matches!(ext.as_str(), "zip" | "rar" | "7z" | "tar" | "gz" | "tgz" | "xz" | "bz2" | "zst")
		})
	}

	fn enter_archive(url: &yazi_shared::url::UrlBuf) -> Result<()> {
		let path = url.loc().as_os().map_or_else(|_| std::path::PathBuf::new(), std::path::PathBuf::from);
		let target_url = url.clone().into_archive("1")?;

		tokio::spawn(async move {
			if yazi_vfs::provider::archive::test_archive_access(&path).await {
				MgrProxy::cd(target_url);
				return;
			}

			let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("archive");
			let mut input = InputProxy::show(InputCfg::password(format!("Password for {filename}:")));

			let Some(Ok(password)) = input.recv().await else { return };
			if password.is_empty() {
				return;
			}

			if yazi_vfs::provider::archive::test_archive_password(&path, &password).await {
				yazi_vfs::provider::archive::set_archive_password(path, password);
				MgrProxy::cd(target_url);
			} else {
				NotifyProxy::push_error("Archive", "Incorrect password");
			}
		});

		Ok(())
	}
}
