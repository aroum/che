use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::{event::ActionCow, url::UrlBuf};

#[derive(Debug)]
pub struct UndoPushCopyPairOpt {
	pub from: Option<UrlBuf>,
	pub to:   Option<UrlBuf>,
}

impl From<ActionCow> for UndoPushCopyPairOpt {
	fn from(mut a: ActionCow) -> Self { Self { from: a.take_any("from"), to: a.take_any("to") } }
}

impl FromLua for UndoPushCopyPairOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for UndoPushCopyPairOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
