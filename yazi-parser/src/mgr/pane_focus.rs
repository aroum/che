use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::event::ActionCow;

#[derive(Debug)]
pub struct PaneFocusOpt {
	pub left: bool,
}

impl From<ActionCow> for PaneFocusOpt {
	fn from(a: ActionCow) -> Self {
		let left = a.bool("left") || a.str(0) == "left" || a.first::<usize>().is_ok_and(|i| i == 0);
		Self { left }
	}
}

impl FromLua for PaneFocusOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for PaneFocusOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
