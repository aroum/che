use anyhow::bail;
use mlua::{FromLua, IntoLua, Lua, Value};
use yazi_fs::FilesOp;
use yazi_shared::{Id, event::ActionCow};

#[derive(Debug)]
pub struct UpdateFilesOpt {
	pub op:   FilesOp,
	pub tabs: Vec<Id>,
}

impl TryFrom<ActionCow> for UpdateFilesOpt {
	type Error = anyhow::Error;

	fn try_from(mut a: ActionCow) -> Result<Self, Self::Error> {
		let Some(op) = a.take_any("op") else {
			bail!("Invalid 'op' in UpdateFilesOpt");
		};

		Ok(Self { op, tabs: a.take_any("tabs").unwrap_or_default() })
	}
}

impl From<FilesOp> for UpdateFilesOpt {
	fn from(op: FilesOp) -> Self { Self { op, tabs: Default::default() } }
}

impl FromLua for UpdateFilesOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err(mlua::Error::runtime("unsupported")) }
}

impl IntoLua for UpdateFilesOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err(mlua::Error::runtime("unsupported")) }
}
