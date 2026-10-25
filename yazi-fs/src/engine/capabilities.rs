use mlua::{FromLua, Lua, Table, Value};

#[derive(Clone, Copy, Debug, Default)]
pub struct Capabilities {
	pub symlink:          bool,
	pub hard_link:        bool,
	pub trash:            bool,
	pub copy_progressive: bool,
}

impl FromLua for Capabilities {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		let t = Table::from_lua(value, lua)?;

		Ok(Self {
			symlink:          t.raw_get("symlink").unwrap_or_default(),
			hard_link:        t.raw_get("hard_link").unwrap_or_default(),
			trash:            t.raw_get("trash").unwrap_or_default(),
			copy_progressive: t.raw_get("copy_progressive").unwrap_or_default(),
		})
	}
}
