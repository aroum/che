use anyhow::Result;
use yazi_core::tab::Folder;
use yazi_fs::FilesOp;
use yazi_macro::{act, render, succ};
use yazi_parser::mgr::UpdateFilesOpt;
use yazi_shared::{data::Data, url::UrlLike};
use yazi_watcher::local::LINKED;

use crate::{Actor, Ctx};

pub struct UpdateFiles;

impl Actor for UpdateFiles {
	type Options = UpdateFilesOpt;

	const NAME: &str = "update_files";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		let linked: Vec<_> = LINKED.read().from_dir(opt.op.cwd()).map(|u| opt.op.chdir(u)).collect();
		let saved_pane = cx.pane;
		let saved_tab = cx.tab;

		let mut revisions = Vec::new();
		for p in 0..2 {
			cx.pane = p;
			let tab_count = cx.mgr.tabs.panes[p].len();
			for t in 0..tab_count {
				cx.tab = t;
				if opt.tabs.is_empty() || opt.tabs.contains(&cx.tab().id) {
					revisions.push((p, t, cx.current().files.revision));
				}
			}
		}

		for op in [opt.op].into_iter().chain(linked) {
			cx.mgr.yanked.apply_op(&op);
			for p in 0..2 {
				cx.pane = p;
				let tab_count = cx.mgr.tabs.panes[p].len();
				for t in 0..tab_count {
					cx.tab = t;
					if opt.tabs.is_empty() || opt.tabs.contains(&cx.tab().id) {
						Self::update_tab(cx, op.clone()).ok();
					}
				}
			}
		}

		render!(cx.mgr.yanked.catchup_revision(false));

		for &(p, t, old_rev) in &revisions {
			cx.pane = p;
			cx.tab = t;
			act!(mgr:hidden, cx).ok();
			act!(mgr:sort, cx).ok();
			if old_rev != cx.current().files.revision {
				act!(mgr:hover, cx)?;
				act!(mgr:update_paged, cx)?;
			}
		}

		cx.pane = saved_pane;
		cx.tab = saved_tab;
		if cx.current().files.revision
			!= revisions
				.iter()
				.find(|&&(p, t, _)| p == saved_pane && t == saved_tab)
				.map_or(0, |&(_, _, r)| r)
		{
			act!(mgr:peek, cx)?;
			act!(mgr:watch, cx)?;
		}
		succ!();
	}
}

impl UpdateFiles {
	fn update_tab(cx: &mut Ctx, op: FilesOp) -> Result<Data> {
		let url = op.cwd();
		cx.tab_mut().selected.apply_op(&op);

		if url == cx.cwd() {
			Self::update_current(cx, op)
		} else if matches!(cx.parent(), Some(p) if *url == p.url) {
			Self::update_parent(cx, op)
		} else if matches!(cx.hovered(), Some(h) if *url == h.url) {
			Self::update_hovered(cx, op)
		} else {
			Self::update_history(cx, op)
		}
	}

	fn update_parent(cx: &mut Ctx, op: FilesOp) -> Result<Data> {
		let tab = cx.tab_mut();

		let urn = tab.current.url.urn();
		let leave = matches!(op, FilesOp::Deleting(_, ref urns) if urns.contains(&urn));

		if let Some(f) = tab.parent.as_mut() {
			render!(f.update_pub(tab.id, op));
			render!(f.hover(urn));
		}

		if leave {
			act!(mgr:leave, cx)?;
		}
		succ!();
	}

	fn update_current(cx: &mut Ctx, op: FilesOp) -> Result<Data> {
		let calc = !matches!(op, FilesOp::Size(..) | FilesOp::Rank(..) | FilesOp::Deleting(..));

		let id = cx.tab().id;
		if !cx.current_mut().update_pub(id, op) {
			succ!();
		}

		if calc {
			cx.tasks.prework_sorted(&cx.current().files);
		}
		succ!();
	}

	fn update_hovered(cx: &mut Ctx, op: FilesOp) -> Result<Data> {
		let (id, url) = (cx.tab().id, op.cwd());
		let folder = cx.tab_mut().history.entry_ref(url).or_insert_with(|| Folder::from(url));

		if folder.update_pub(id, op) {
			act!(mgr:peek, cx, true)?;
		}
		succ!();
	}

	fn update_history(cx: &mut Ctx, op: FilesOp) -> Result<Data> {
		let tab = &mut cx.tab_mut();
		let leave = tab.parent.as_ref().and_then(|f| f.url.parent().map(|p| (p, f.url.urn()))).is_some_and(
			|(p, n)| matches!(op, FilesOp::Deleting(ref parent, ref urns) if *parent == p && urns.contains(&n)),
		);

		tab
			.history
			.entry_ref(op.cwd())
			.or_insert_with(|| Folder::from(op.cwd()))
			.update_pub(tab.id, op);

		if leave {
			act!(mgr:leave, cx)?;
		}
		succ!();
	}
}
