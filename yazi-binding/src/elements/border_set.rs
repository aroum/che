use mlua::{FromLua, Lua, LuaSerdeExt, UserData, Value};
use ratatui::symbols::border;
use serde::{Deserialize, Deserializer, de};
use unicode_width::UnicodeWidthStr;

use super::BorderType;

#[derive(Clone, Debug, Default, Deserialize)]
pub struct BorderSet {
	#[serde(default, rename = "as", alias = "0")]
	pub r#as: BorderType,

	#[serde(default, deserialize_with = "BorderSet::deserialize_symbol")]
	pub tl: String,
	#[serde(default, deserialize_with = "BorderSet::deserialize_symbol")]
	pub tc: String,
	#[serde(default, deserialize_with = "BorderSet::deserialize_symbol")]
	pub tr: String,
	#[serde(default, deserialize_with = "BorderSet::deserialize_symbol")]
	pub cl: String,
	#[serde(default, deserialize_with = "BorderSet::deserialize_symbol")]
	pub cr: String,
	#[serde(default, deserialize_with = "BorderSet::deserialize_symbol")]
	pub bl: String,
	#[serde(default, deserialize_with = "BorderSet::deserialize_symbol")]
	pub bc: String,
	#[serde(default, deserialize_with = "BorderSet::deserialize_symbol")]
	pub br: String,
}

impl From<BorderType> for BorderSet {
	fn from(r#as: BorderType) -> Self { Self { r#as, ..Default::default() } }
}

impl BorderSet {
	pub fn as_set(&self) -> border::Set<'_> {
		let mut set = self.r#as.to_border_set();
		if !self.tl.is_empty() { set.top_left = &self.tl; }
		if !self.tc.is_empty() { set.horizontal_top = &self.tc; }
		if !self.tr.is_empty() { set.top_right = &self.tr; }
		if !self.cl.is_empty() { set.vertical_left = &self.cl; }
		if !self.cr.is_empty() { set.vertical_right = &self.cr; }
		if !self.bl.is_empty() { set.bottom_left = &self.bl; }
		if !self.bc.is_empty() { set.horizontal_bottom = &self.bc; }
		if !self.br.is_empty() { set.bottom_right = &self.br; }
		set
	}

	fn deserialize_symbol<'de, D>(de: D) -> Result<String, D::Error>
	where
		D: Deserializer<'de>,
	{
		let s = String::deserialize(de)?;
		if s.chars().any(char::is_control) || (!s.is_empty() && s.width() != 1) {
			return Err(de::Error::custom("expected an empty or 1-width border symbol"));
		}
		Ok(s)
	}
}

impl FromLua for BorderSet {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		match value {
			Value::String(_) => Ok(lua.from_value::<BorderType>(value)?.into()),
			Value::UserData(ud) => Ok(ud.borrow::<Self>()?.clone()),
			Value::Table(_) => lua.from_value(value),
			_ => Err(mlua::Error::FromLuaConversionError {
				from: value.type_name(),
				to: "BorderSet".to_string(),
				message: None,
			}),
		}
	}
}

impl UserData for BorderSet {}
