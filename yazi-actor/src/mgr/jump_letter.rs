use anyhow::Result;
use yazi_macro::{render, succ};
use yazi_parser::mgr::JumpLetterOpt;
use yazi_shared::{data::Data, path::AsPath};

use crate::{Actor, Ctx};

pub struct JumpLetter;

impl Actor for JumpLetter {
	type Options = JumpLetterOpt;

	const NAME: &str = "jump_letter";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		if opt.ch == '\0' {
			succ!();
		}

		let tab = cx.tab_mut();
		tab.last_jump_char = Some(opt.ch);

		let ch_lower = opt.ch.to_lowercase().to_string();
		let matching: Vec<usize> = tab
			.current
			.files
			.iter()
			.enumerate()
			.filter_map(|(idx, file)| {
				let name = file.name().map(|n| n.to_string_lossy().to_lowercase()).unwrap_or_default();
				if name.starts_with(&ch_lower) {
					Some(idx)
				} else {
					None
				}
			})
			.collect();

		if matching.is_empty() {
			succ!();
		}

		let current_cursor = tab.current.cursor;
		let target_idx = if let Some(pos) = matching.iter().position(|&idx| idx == current_cursor) {
			matching[(pos + 1) % matching.len()]
		} else {
			matching[0]
		};

		let urn = tab.current.files[target_idx].urn().to_owned();
		succ!(render!(tab.current.hover(urn.as_path())))
	}
}
