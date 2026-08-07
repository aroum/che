use anyhow::bail;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use tokio::sync::mpsc::UnboundedSender;
use yazi_config::popup::InputCfg;
use yazi_shared::event::ActionCow;
use yazi_widgets::input::InputError;

#[derive(Debug)]
pub struct ShowOpt {
	pub cfg: InputCfg,
	pub tx:  UnboundedSender<Result<String, InputError>>,
}

impl TryFrom<ActionCow> for ShowOpt {
	type Error = anyhow::Error;

	fn try_from(mut a: ActionCow) -> Result<Self, Self::Error> {
		let Some(cfg) = a.take_any("cfg") else {
			bail!("Invalid 'cfg' in ShowOpt");
		};

		let Some(tx) = a.take_any("tx") else {
			bail!("Invalid 'tx' in ShowOpt");
		};

		Ok(Self { cfg, tx })
	}
}

impl From<Box<Self>> for ShowOpt {
	fn from(value: Box<Self>) -> Self { *value }
}

impl FromLua for ShowOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for ShowOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
