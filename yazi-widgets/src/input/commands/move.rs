use anyhow::Result;
use yazi_macro::{render, succ};
use yazi_shared::data::Data;

use crate::input::{Input, op::InputOp, parser::MoveOpt, snap::InputSnap};

impl Input {
	pub fn r#move(&mut self, opt: MoveOpt) -> Result<Data> {
		let snap = self.snap();
		if opt.in_operating && snap.op == InputOp::None {
			succ!();
		}

		let o_cur = snap.cursor;
		let n_cur = opt.step.add(&snap.value, snap.cursor);

		if opt.visual {
			let snap = self.snap_mut();
			if !matches!(snap.op, InputOp::Select(_)) {
				snap.op = InputOp::Select(o_cur);
			}
			render!(self.handle_op(n_cur, false));
		} else {
			let mut handled = false;
			if let InputOp::Select(start) = snap.op {
				let snap = self.snap_mut();
				snap.op = InputOp::None;
				match opt.step {
					crate::input::parser::MoveOptStep::Offset(n) if n > 0 => {
						snap.cursor = start.max(o_cur);
						handled = true;
					}
					crate::input::parser::MoveOptStep::Offset(n) if n < 0 => {
						snap.cursor = start.min(o_cur);
						handled = true;
					}
					_ => {}
				}
			}
			if !handled {
				render!(self.handle_op(n_cur, false));
			} else {
				render!();
			}
		}

		let (limit, snap) = (self.limit, self.snap_mut());
		if snap.value.is_empty() {
			succ!(snap.offset = 0);
		}

		let (o_off, scrolloff) = (snap.offset, 5.min(limit / 2));
		snap.offset = if n_cur <= o_cur {
			let it = snap.slice(0..n_cur).chars().rev().map(|c| if snap.obscure { '•' } else { c });
			let pad = InputSnap::find_window(it, 0, scrolloff).end;

			if n_cur >= o_off { snap.offset.min(n_cur - pad) } else { n_cur - pad }
		} else {
			let count = snap.count();

			let it = snap.slice(n_cur..count).chars().map(|c| if snap.obscure { '•' } else { c });
			let pad = InputSnap::find_window(it, 0, scrolloff + snap.mode.delta()).end;

			let it = snap.slice(0..n_cur + pad).chars().rev().map(|c| if snap.obscure { '•' } else { c });
			let max = InputSnap::find_window(it, 0, limit).end;

			if snap.width(o_off..n_cur) < limit as u16 {
				snap.offset.max(n_cur + pad - max)
			} else {
				n_cur + pad - max
			}
		};
		succ!();
	}
}
