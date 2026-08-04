yazi_macro::mod_pub!(actions);

yazi_macro::mod_flat!(args boot);

use clap::Parser;
use yazi_shared::RoCell;

pub static ARGS: RoCell<Args> = RoCell::new();
pub static BOOT: RoCell<Boot> = RoCell::new();

pub fn init_args() {
	ARGS.with(<_>::parse);
	if ARGS.logging {
		unsafe {
			std::env::set_var("CHE_LOG", "debug");
		}
	}
}

pub fn init() {
	BOOT.init(<_>::from(&*ARGS));

	actions::Actions::act(&ARGS);

	// Expose the path to the sibling `ch` binary so Lua plugins can invoke it.
	if let Ok(exe) = std::env::current_exe() {
		if let Some(dir) = exe.parent() {
			let ch = dir.join("ch");
			// SAFETY: single-threaded init before any tokio threads are spawned.
			unsafe {
				std::env::set_var("CHE_CH_PATH", &ch);
				std::env::set_var("GEZI_GEZ_PATH", &ch);
			}
		}
	}
}

pub fn init_default() {
	ARGS.with(<_>::default);
	BOOT.with(<_>::default);
}
