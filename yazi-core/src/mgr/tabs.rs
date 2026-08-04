use std::collections::VecDeque;

use yazi_dds::Pubsub;
use yazi_fs::File;
use yazi_macro::err;
use yazi_shared::Id;

use crate::tab::{Folder, Tab};

const MAX_CLOSED_TABS: usize = 10;

pub struct Pane {
	pub cursor: usize,
	pub items:  Vec<Tab>,
	pub closed: VecDeque<Tab>,
}

impl Default for Pane {
	fn default() -> Self {
		Self { cursor: 0, items: vec![Default::default()], closed: VecDeque::new() }
	}
}

impl Pane {
	pub fn push_closed(&mut self, mut tab: Tab) {
		tab.shutdown();
		if self.closed.len() >= MAX_CLOSED_TABS {
			self.closed.pop_front();
		}
		self.closed.push_back(tab);
	}

	pub fn pop_closed(&mut self) -> Option<Tab> {
		self.closed.pop_back()
	}

	#[inline]
	pub fn active(&self) -> &Tab { &self.items[self.cursor] }

	#[inline]
	pub fn active_mut(&mut self) -> &mut Tab { &mut self.items[self.cursor] }

	#[inline]
	pub fn len(&self) -> usize { self.items.len() }

	pub fn set_idx(&mut self, idx: usize) {
		if let Some(active) = self.items.get_mut(self.cursor) {
			active.preview.reset_image();
		}
		self.cursor = idx;
		err!(Pubsub::pub_after_tab(self.active().id));
	}
}

pub struct Tabs {
	pub active_pane: usize,
	pub panes: [Pane; 2],
	pub single_pane: bool,
	pub preview_pane: bool,
}

impl Default for Tabs {
	fn default() -> Self {
		Self {
			active_pane: 0,
			panes: [Pane::default(), Pane::default()],
			single_pane: false,
			preview_pane: false,
		}
	}
}

impl Tabs {
	pub fn set_idx(&mut self, idx: usize) {
		self.panes[self.active_pane].set_idx(idx);
	}

	pub fn set_active_pane(&mut self, idx: usize) {
		if let Some(pane) = self.panes.get_mut(self.active_pane) {
			pane.active_mut().preview.reset_image();
		}
		self.active_pane = idx;
		err!(Pubsub::pub_after_tab(self.active().id));
	}

	pub fn find_tab(&self, id: Id) -> Option<(usize, usize)> {
		for (pi, pane) in self.panes.iter().enumerate() {
			if let Some(ti) = pane.items.iter().position(|t| t.id == id) {
				return Some((pi, ti));
			}
		}
		None
	}
}

impl Tabs {
	#[inline]
	pub fn active(&self) -> &Tab { self.panes[self.active_pane].active() }

	#[inline]
	pub fn other(&self) -> &Tab { self.panes[1 - self.active_pane].active() }

	#[inline]
	pub(super) fn active_mut(&mut self) -> &mut Tab {
		self.panes[self.active_pane].active_mut()
	}

	#[inline]
	pub fn other_mut(&mut self) -> &mut Tab {
		self.panes[1 - self.active_pane].active_mut()
	}

	#[inline]
	pub fn active_pane_ref(&self) -> &Pane { &self.panes[self.active_pane] }

	#[inline]
	pub fn other_pane_ref(&self) -> &Pane { &self.panes[1 - self.active_pane] }

	#[inline]
	pub fn active_pane_mut(&mut self) -> &mut Pane { &mut self.panes[self.active_pane] }

	#[inline]
	pub fn other_pane_mut(&mut self) -> &mut Pane { &mut self.panes[1 - self.active_pane] }

	#[inline]
	pub fn parent(&self) -> Option<&Folder> { self.active().parent.as_ref() }

	#[inline]
	pub fn current(&self) -> &Folder { &self.active().current }

	#[inline]
	pub fn hovered(&self) -> Option<&File> { self.current().hovered() }

	pub fn all_tabs(&self) -> impl Iterator<Item = &Tab> {
		self.panes.iter().flat_map(|p| p.items.iter())
	}

	pub fn all_tabs_mut(&mut self) -> impl Iterator<Item = &mut Tab> {
		self.panes.iter_mut().flat_map(|p| p.items.iter_mut())
	}
}
