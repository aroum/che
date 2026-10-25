use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::event::ActionCow;

#[derive(Clone, Copy, Debug, Default)]
pub struct VisualArrowOpt {
	pub step: isize,
}

impl From<ActionCow> for VisualArrowOpt {
	fn from(a: ActionCow) -> Self { Self { step: a.first().unwrap_or(0) } }
}

impl From<isize> for VisualArrowOpt {
	fn from(step: isize) -> Self { Self { step } }
}

impl FromLua for VisualArrowOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for VisualArrowOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
