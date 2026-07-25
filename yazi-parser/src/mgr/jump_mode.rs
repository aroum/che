use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::event::ActionCow;

#[derive(Clone, Debug, Default)]
pub struct JumpModeOpt {
	pub state: Option<bool>,
}

impl From<ActionCow> for JumpModeOpt {
	fn from(mut a: ActionCow) -> Self {
		Self {
			state: a.take_any("state"),
		}
	}
}

impl From<()> for JumpModeOpt {
	fn from(_: ()) -> Self {
		Self { state: None }
	}
}

impl From<bool> for JumpModeOpt {
	fn from(state: bool) -> Self {
		Self { state: Some(state) }
	}
}

impl From<Option<bool>> for JumpModeOpt {
	fn from(state: Option<bool>) -> Self {
		Self { state }
	}
}

impl FromLua for JumpModeOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for JumpModeOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
