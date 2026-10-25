use mlua::{Lua, Table};
use serde::Deserialize;

#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BorderType {
	#[default]
	Plain,
	Rounded,
	Double,
	Thick,
	QuadrantInside,
	QuadrantOutside,
}

impl BorderType {
	pub fn to_border_set(self) -> ratatui::symbols::border::Set<'static> {
		match self {
			Self::Plain => ratatui::symbols::border::PLAIN,
			Self::Rounded => ratatui::symbols::border::ROUNDED,
			Self::Double => ratatui::symbols::border::DOUBLE,
			Self::Thick => ratatui::symbols::border::THICK,
			Self::QuadrantInside => ratatui::symbols::border::QUADRANT_INSIDE,
			Self::QuadrantOutside => ratatui::symbols::border::QUADRANT_OUTSIDE,
		}
	}

	pub fn compose(lua: &Lua) -> mlua::Result<Table> {
		lua.create_table_from([
			("PLAIN", 0),
			("ROUNDED", 1),
			("DOUBLE", 2),
			("THICK", 3),
			("QUADRANT_INSIDE", 4),
			("QUADRANT_OUTSIDE", 5),
		])
	}
}

impl From<u8> for BorderType {
	fn from(value: u8) -> Self {
		match value {
			1 => Self::Rounded,
			2 => Self::Double,
			3 => Self::Thick,
			4 => Self::QuadrantInside,
			5 => Self::QuadrantOutside,
			_ => Self::Plain,
		}
	}
}
