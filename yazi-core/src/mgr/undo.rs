pub use yazi_shared::UndoOp;

#[derive(Clone, Debug)]
pub struct UndoEntry {
	pub op: UndoOp,
}

const MAX_UNDO_ENTRIES: usize = 20;

#[derive(Debug, Default)]
pub struct UndoManager {
	undo_stack: Vec<UndoEntry>,
	redo_stack: Vec<UndoEntry>,
}

impl UndoManager {
	pub fn push(&mut self, op: UndoOp) {
		self.redo_stack.clear();
		if self.undo_stack.len() >= MAX_UNDO_ENTRIES {
			self.undo_stack.remove(0);
		}
		self.undo_stack.push(UndoEntry { op });
	}

	pub fn push_trash_pair(
		&mut self,
		original: yazi_shared::url::UrlBuf,
		trash_path: yazi_shared::url::UrlBuf,
	) {
		if let Some(entry) = self.undo_stack.last_mut() {
			if let UndoOp::Trash { ref mut pairs } = entry.op {
				pairs.push((original, trash_path));
				return;
			}
		}
		self.redo_stack.clear();
		if self.undo_stack.len() >= MAX_UNDO_ENTRIES {
			self.undo_stack.remove(0);
		}
		self.undo_stack.push(UndoEntry { op: UndoOp::Trash { pairs: vec![(original, trash_path)] } });
	}

	pub fn push_copy_pair(&mut self, from: yazi_shared::url::UrlBuf, to: yazi_shared::url::UrlBuf) {
		if let Some(entry) = self.undo_stack.last_mut() {
			if let UndoOp::Copy { ref mut pairs, .. } = entry.op {
				pairs.push((from, to));
			}
		}
	}

	pub fn push_move_pair(&mut self, from: yazi_shared::url::UrlBuf, to: yazi_shared::url::UrlBuf) {
		if let Some(entry) = self.undo_stack.last_mut() {
			if let UndoOp::Move { ref mut pairs, .. } = entry.op {
				pairs.push((from, to));
			}
		}
	}

	pub fn push_copy_overwritten(
		&mut self,
		original: yazi_shared::url::UrlBuf,
		trash_path: yazi_shared::url::UrlBuf,
	) {
		if let Some(entry) = self.undo_stack.last_mut() {
			if let UndoOp::Copy { ref mut overwritten, .. } = entry.op {
				overwritten.push((original, trash_path));
			}
		}
	}

	pub fn push_move_overwritten(
		&mut self,
		original: yazi_shared::url::UrlBuf,
		trash_path: yazi_shared::url::UrlBuf,
	) {
		if let Some(entry) = self.undo_stack.last_mut() {
			if let UndoOp::Move { ref mut overwritten, .. } = entry.op {
				overwritten.push((original, trash_path));
			}
		}
	}

	pub fn undo_stack_last_mut(&mut self) -> Option<&mut UndoEntry> { self.undo_stack.last_mut() }

	pub fn undo(&mut self) -> Option<UndoEntry> {
		let entry = self.undo_stack.pop()?;
		self.redo_stack.push(entry.clone());
		Some(entry)
	}

	pub fn redo(&mut self) -> Option<UndoEntry> {
		let entry = self.redo_stack.pop()?;
		self.undo_stack.push(entry.clone());
		Some(entry)
	}
}
