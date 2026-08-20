use std::collections::VecDeque;

use yazi_dds::Pubsub;
use yazi_macro::err;

use crate::tab::Tab;

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
