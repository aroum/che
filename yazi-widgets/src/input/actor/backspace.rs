use anyhow::Result;
use yazi_macro::{act, render, succ};
use yazi_shared::data::Data;

use crate::input::{Input, parser::BackspaceOpt};

impl Input {
	pub fn backspace(&mut self, opt: BackspaceOpt) -> Result<Data> {
		let snap = self.snap_mut();
		if let crate::input::op::InputOp::Select(start) = snap.op {
			let end = snap.cursor;
			let range = if start < end { start..end } else { end..start };
			let start_idx = snap.idx(range.start).unwrap();
			let end_idx = snap.idx(range.end).unwrap();
			snap.value.drain(start_idx..end_idx);
			snap.cursor = range.start;
			snap.op = crate::input::op::InputOp::None;
			self.flush_value();
			succ!(render!());
		}

		if !opt.under && snap.cursor < 1 {
			succ!();
		} else if opt.under && snap.cursor >= snap.count() {
			succ!();
		}

		if opt.under {
			snap.value.remove(snap.idx(snap.cursor).unwrap());
			act!(r#move, self)?;
		} else {
			snap.value.remove(snap.idx(snap.cursor - 1).unwrap());
			act!(r#move, self, -1)?;
		}

		self.flush_value();
		succ!(render!());
	}
}
