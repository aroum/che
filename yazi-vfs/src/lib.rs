#![allow(clippy::all)]

yazi_macro::mod_pub!(engine);

pub use engine as provider;

yazi_macro::mod_flat!(cha file files fns op);

pub fn init() {
	engine::init();
}
