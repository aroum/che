use tokio::sync::mpsc::UnboundedSender;
use yazi_config::{popup::Position, YAZI};
use yazi_widgets::Scrollable;

#[derive(Default)]
pub struct Pick {
	pub title:    String,
	pub items:    Vec<String>,
	pub position: Position,

	pub offset:   usize,
	pub cursor:   usize,
	pub callback: Option<UnboundedSender<Option<usize>>>,

	pub visible: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PickJumpResult {
	None,
	Moved,
	Submit,
}

impl Pick {
	pub fn title(&self) -> &str { &self.title }

	pub fn window(&self) -> impl Iterator<Item = (usize, &str)> {
		self.items.iter().map(AsRef::as_ref).enumerate().skip(self.offset).take(self.limit())
	}

	pub fn jump_letter(&mut self, ch: char) -> PickJumpResult {
		if self.items.is_empty() {
			return PickJumpResult::None;
		}

		let ch_lower = ch.to_lowercase().to_string();
		let matching: Vec<usize> = self
			.items
			.iter()
			.enumerate()
			.filter_map(|(idx, item)| {
				let name_lower = item.to_lowercase();
				let clean_name = if let Some(s) = name_lower.strip_prefix("volume: ") {
					s
				} else if let Some(s) = name_lower.strip_prefix("drive ") {
					s
				} else if let Some(s) = name_lower.strip_prefix("mount: ") {
					s
				} else {
					&name_lower
				};

				if clean_name.starts_with(&ch_lower) || name_lower.starts_with(&ch_lower) {
					Some(idx)
				} else {
					None
				}
			})
			.collect();

		if matching.is_empty() {
			return PickJumpResult::None;
		}

		if matching.len() == 1 {
			self.cursor = matching[0];
			return PickJumpResult::Submit;
		}

		let pos = matching
			.iter()
			.position(|&idx| idx == self.cursor)
			.map(|i| (i + 1) % matching.len())
			.unwrap_or(0);

		let target = matching[pos];
		self.cursor = target;

		let limit = self.limit();
		if self.cursor < self.offset {
			self.offset = self.cursor;
		} else if self.cursor >= self.offset + limit {
			self.offset = self.cursor + 1 - limit;
		}

		PickJumpResult::Moved
	}
}

impl Scrollable for Pick {
	fn total(&self) -> usize { self.items.len() }

	fn limit(&self) -> usize {
		self.position.offset.height.saturating_sub(YAZI.pick.border()) as usize
	}

	fn cursor_mut(&mut self) -> &mut usize { &mut self.cursor }

	fn offset_mut(&mut self) -> &mut usize { &mut self.offset }
}
