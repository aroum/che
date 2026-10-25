#![allow(clippy::all, unused)]

yazi_macro::mod_pub!(elements external fs isolate loader process pubsub runtime theme utils);

pub use isolate::slim_lua_env;

yazi_macro::mod_flat!(lua);

pub fn init() -> anyhow::Result<()> {
	crate::loader::init();
	crate::init_lua()?;
	Ok(())
}
