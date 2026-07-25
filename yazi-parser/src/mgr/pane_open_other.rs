use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::event::ActionCow;

#[derive(Debug)]
pub struct PaneOpenOtherOpt;

impl From<ActionCow> for PaneOpenOtherOpt {
	fn from(_: ActionCow) -> Self { Self }
}

impl FromLua for PaneOpenOtherOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for PaneOpenOtherOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
