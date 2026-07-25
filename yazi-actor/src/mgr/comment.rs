use anyhow::Result;
use yazi_config::popup::InputCfg;
use yazi_macro::succ;
use yazi_parser::mgr::CommentOpt;
use yazi_proxy::InputProxy;
use yazi_shared::{data::Data, url::UrlLike};

use crate::{Actor, Ctx};

pub struct Comment;

impl Actor for Comment {
	type Options = CommentOpt;

	const NAME: &str = "comment";

	fn act(cx: &mut Ctx, _opt: Self::Options) -> Result<Data> {
		let Some(hovered) = cx.hovered() else { succ!() };
		let Some(path) = hovered.url.loc().as_os().ok().map(|p| p.to_path_buf()) else { succ!() };

		let current_desc = yazi_vfs::provider::read_description(&path).unwrap_or_default();

		let mut cfg = InputCfg::rename().with_value(current_desc);
		cfg.title = "Edit Comment:".to_owned();

		let mut input = InputProxy::show(cfg);

		tokio::spawn(async move {
			if let Some(Ok(new_desc)) = input.recv().await {
				yazi_vfs::provider::write_description(&path, &new_desc);
			}
		});

		succ!()
	}
}
