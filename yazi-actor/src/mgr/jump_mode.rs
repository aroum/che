use anyhow::Result;
use yazi_macro::{render, succ};
use yazi_parser::mgr::JumpModeOpt;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct JumpMode;

impl Actor for JumpMode {
	type Options = JumpModeOpt;

	const NAME: &str = "jump_mode";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		let tab = cx.tab_mut();
		let target_state = opt.state.unwrap_or(!tab.jump_mode);
		tab.jump_mode = target_state;
		if !tab.jump_mode {
			tab.last_jump_char = None;
		}
		succ!(render!())
	}
}
