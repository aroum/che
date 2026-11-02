use anyhow::Result;
use yazi_macro::{act, succ};
use yazi_parser::{VoidOpt, mgr::CdSource};
use yazi_shared::{data::Data, url::UrlLike};

use crate::{Actor, Ctx};

pub struct Leave;

impl Actor for Leave {
	type Options = VoidOpt;

	const NAME: &str = "leave";

	fn act(cx: &mut Ctx, _: Self::Options) -> Result<Data> {
		let url = Self::target_url(cx.cwd(), cx.hovered());

		let Some(mut url) = url else { succ!() };
		if url.is_search() || (url.is_archive() && url.parent().is_none()) {
			url = url.parent().unwrap_or(url);
		}

		act!(mgr:cd, cx, (url, CdSource::Leave))
	}
}

impl Leave {
	pub fn target_url<'a>(
		cwd: &'a yazi_shared::url::UrlBuf,
		hovered: Option<&'a yazi_fs::File>,
	) -> Option<yazi_shared::url::Url<'a>> {
		if cwd.is_archive() {
			cwd.parent()
		} else {
			hovered
				.filter(|h| !h.is_upparent)
				.and_then(|h| h.url.parent())
				.filter(|u| u != cwd)
				.or_else(|| cwd.parent())
		}
	}
}

#[cfg(test)]
mod tests {
	use yazi_fs::File;
	use yazi_shared::url::{AsUrl, UrlBuf};

	use super::*;

	#[test]
	fn test_leave_target_url_with_upparent() {
		let cwd: UrlBuf = "/home/user/project/src".parse().unwrap();
		let parent: UrlBuf = "/home/user/project".parse().unwrap();

		// Case 1: Hovered is the virtual ".." entry
		let upparent = File::make_upparent(parent.clone());
		assert!(upparent.is_upparent);
		let target = Leave::target_url(&cwd, Some(&upparent));
		assert_eq!(
			target,
			Some(parent.as_url()),
			"Hovering on '..' should leave to cwd's parent, not upper-upper parent"
		);

		// Case 2: Hovered is a normal file inside cwd
		let file = File::from_dummy(cwd.try_join("main.rs").unwrap(), None);
		assert!(!file.is_upparent);
		let target = Leave::target_url(&cwd, Some(&file));
		assert_eq!(
			target,
			Some(parent.as_url()),
			"Hovering on a regular file should leave to cwd's parent"
		);

		// Case 3: Hovered is a search result item from another directory
		let other_dir: UrlBuf = "/home/user/other".parse().unwrap();
		let search_item = File::from_dummy(other_dir.try_join("doc.txt").unwrap(), None);
		let target = Leave::target_url(&cwd, Some(&search_item));
		assert_eq!(
			target,
			Some(other_dir.as_url()),
			"Hovering on a search item should leave to that item's parent"
		);

		// Case 4: No hovered item
		let target = Leave::target_url(&cwd, None);
		assert_eq!(target, Some(parent.as_url()), "When nothing is hovered, leave to cwd's parent");
	}
}
