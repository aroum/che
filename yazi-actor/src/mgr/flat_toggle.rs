use anyhow::Result;
use yazi_macro::{act, render, succ};
use yazi_parser::mgr::{SearchOpt, SearchOptVia};
use yazi_shared::{data::Data, url::UrlLike};

use crate::{Actor, Ctx};

pub struct FlatToggle;

impl Actor for FlatToggle {
	type Options = yazi_parser::mgr::FlatToggleOpt;

	const NAME: &str = "flat_toggle";

	fn act(cx: &mut Ctx, _: Self::Options) -> Result<Data> {
		let in_flat_view = cx.tab().search.is_some() || cx.tab().cwd().is_search();
		if in_flat_view {
			// Grab the real filesystem path of the hovered file (converts search URL → regular URL).
			// We must do this before aborting the handle / changing cwd.
			let hovered_regular = cx
				.tab()
				.hovered()
				.and_then(|f| f.url.to_regular().ok());

			if let Some(handle) = cx.tab_mut().search.take() {
				handle.abort();
			}

			if let Some(url) = hovered_regular {
				act!(mgr:reveal, cx, url)?;
			} else {
				act!(mgr:escape_search, cx)?;
			}
		} else {
			act!(
				mgr:search_do,
				cx,
				SearchOpt {
					via: SearchOptVia::Fd,
					subject: "".into(),
					args: vec![],
					args_raw: "".into(),
					r#in: None,
				}
			)?;
		}
		succ!(render!())
	}
}
