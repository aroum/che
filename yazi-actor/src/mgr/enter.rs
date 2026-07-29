use anyhow::Result;
use yazi_macro::{act, succ};
use yazi_parser::{VoidOpt, mgr::CdSource};
use yazi_shared::{data::Data, url::UrlLike};

use crate::{Actor, Ctx};

pub struct Enter;

impl Actor for Enter {
	type Options = VoidOpt;

	const NAME: &str = "enter";

	fn act(cx: &mut Ctx, _: Self::Options) -> Result<Data> {
		let Some(h) = cx.hovered() else { succ!() };

		let is_archive_ext = h.url.ext().is_some_and(|ext| {
			let ext = ext.to_string_lossy().to_lowercase();
			matches!(ext.as_str(), "zip" | "rar" | "7z" | "tar" | "gz" | "tgz" | "xz" | "bz2" | "zst")
		});

		if h.is_dir() {
			let url = if h.url.is_search() { h.url.to_regular()? } else { h.url.clone() };
			act!(mgr:cd, cx, (url, CdSource::Enter))
		} else if is_archive_ext && yazi_config::YAZI.mgr.archive_vfs.get() {
			let url = h.url.clone().into_archive("1")?;
			act!(mgr:cd, cx, (url, CdSource::Enter))
		} else {
			succ!()
		}
	}
}
