use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::{event::ActionCow, url::UrlBuf};

#[derive(Debug)]
pub struct UndoPushMovePairOpt {
	pub from: Option<UrlBuf>,
	pub to:   Option<UrlBuf>,
}

impl From<ActionCow> for UndoPushMovePairOpt {
	fn from(mut a: ActionCow) -> Self { Self { from: a.take_any("from"), to: a.take_any("to") } }
}

impl FromLua for UndoPushMovePairOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for UndoPushMovePairOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
