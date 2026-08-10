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
		let exe = std::fs::canonicalize(&exe).unwrap_or(exe);
		if let Some(dir) = exe.parent() {
			#[cfg(target_os = "windows")]
			let ch = dir.join("ch.exe");
			#[cfg(not(target_os = "windows"))]
			let ch = dir.join("ch");

			if ch.is_file() {
				// SAFETY: single-threaded init before any tokio threads are spawned.
				unsafe {
					std::env::set_var("CHE_CH_PATH", &ch);
					std::env::set_var("GEZI_GEZ_PATH", &ch);
				}
			}
		}
	}
}

pub fn init_default() {
	ARGS.with(<_>::default);
	BOOT.with(<_>::default);
}
