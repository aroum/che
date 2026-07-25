use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::event::ActionCow;

#[derive(Clone, Debug, Default)]
pub struct JumpLetterOpt {
	pub ch: char,
}

impl From<ActionCow> for JumpLetterOpt {
	fn from(mut a: ActionCow) -> Self {
		let ch = a
			.take_any::<char>("ch")
			.or_else(|| a.take_any::<String>("ch").and_then(|s| s.chars().next()))
			.unwrap_or('\0');
		Self { ch }
	}
}

impl From<char> for JumpLetterOpt {
	fn from(ch: char) -> Self {
		Self { ch }
	}
}

impl FromLua for JumpLetterOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for JumpLetterOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
