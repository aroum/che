use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::event::ActionCow;

#[derive(Clone, Debug)]
pub struct CommentOpt;

impl From<ActionCow> for CommentOpt {
	fn from(_a: ActionCow) -> Self {
		Self
	}
}

impl FromLua for CommentOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for CommentOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
