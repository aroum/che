yazi_macro::mod_pub!(cha engine error file mounts path);

pub use engine as provider;
pub use file::*;

yazi_macro::mod_flat!(cwd files filter fns hash normalizer op scheme sorter sorting splatter stage url xdg);

pub fn init() {
	CWD.init(<_>::default());

	mounts::init();
}
