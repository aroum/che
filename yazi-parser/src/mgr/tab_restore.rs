use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::event::ActionCow;

#[derive(Debug, Default)]
pub struct TabRestoreOpt;

impl From<ActionCow> for TabRestoreOpt {
	fn from(_: ActionCow) -> Self { Self }
}

impl FromLua for TabRestoreOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for TabRestoreOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
