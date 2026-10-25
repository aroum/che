use std::borrow::Cow;

use hashbrown::HashMap;
use mlua::{ExternalError, Lua, Table, Value};

use crate::{
	data::{Data, DataKey},
	path::PathLike,
	replace_cow,
	url::UrlLike,
};

pub struct Sendable;

impl Sendable {
	pub fn table_to_args(lua: &Lua, t: Table) -> mlua::Result<HashMap<DataKey, Data>> {
		let mut args = HashMap::with_capacity(t.raw_len());
		for pair in t.pairs::<Value, Value>() {
			let (k, v) = pair?;
			match k {
				Value::Integer(i) if i > 0 => {
					args.insert(DataKey::Integer(i - 1), Self::value_to_data(lua, v)?);
				}
				Value::String(s) => {
					args.insert(
						DataKey::String(Cow::Owned(s.to_str()?.replace('_', "-"))),
						Self::value_to_data(lua, v)?,
					);
				}
				_ => return Err("invalid key in Action".into_lua_err()),
			}
		}
		Ok(args)
	}

	pub fn args_to_table(lua: &Lua, args: HashMap<DataKey, Data>) -> mlua::Result<Table> {
		let seq_len = args.keys().filter(|&k| k.is_integer()).count();
		let tbl = lua.create_table_with_capacity(seq_len, args.len() - seq_len)?;
		for (k, v) in args {
			match k {
				DataKey::Integer(i) => tbl.raw_set(i + 1, Self::data_to_value(lua, v)?),
				DataKey::String(s) => {
					tbl.raw_set(replace_cow(s, "-", "_"), Self::data_to_value(lua, v)?)
				}
				_ => Err("invalid key in Data".into_lua_err()),
			}?;
		}
		Ok(tbl)
	}

	pub fn args_to_table_ref(lua: &Lua, args: &HashMap<DataKey, Data>) -> mlua::Result<Table> {
		let seq_len = args.keys().filter(|&k| k.is_integer()).count();
		let tbl = lua.create_table_with_capacity(seq_len, args.len() - seq_len)?;
		for (k, v) in args {
			match k {
				DataKey::Integer(i) => tbl.raw_set(i + 1, Self::data_to_value_ref(lua, v)?),
				DataKey::String(s) => {
					tbl.raw_set(replace_cow(&**s, "-", "_"), Self::data_to_value_ref(lua, v)?)
				}
				_ => Err("invalid key in Data".into_lua_err()),
			}?;
		}
		Ok(tbl)
	}

	pub fn value_to_data(lua: &Lua, value: Value) -> mlua::Result<Data> {
		Ok(match value {
			Value::Nil => Data::Nil,
			Value::Boolean(b) => Data::Boolean(b),
			Value::Integer(i) => Data::Integer(i),
			Value::Number(n) => Data::Number(n),
			Value::String(b) => {
				if let Ok(s) = b.to_str() {
					Data::String(s.to_owned().into())
				} else {
					Data::Bytes(b.as_bytes().to_owned())
				}
			}
			Value::Table(t) => {
				let (mut i, mut map) = (1, HashMap::with_capacity(t.raw_len()));
				for result in t.pairs::<Value, Value>() {
					let (k, v) = result?;
					let k = Self::value_to_key(k)?;
					if k == DataKey::Integer(i) {
						i += 1;
					}
					map.insert(k, Self::value_to_data(lua, v)?);
				}

				if map.len() == i as usize - 1 {
					Data::List((1..i).map(|i| map.remove(&DataKey::Integer(i)).unwrap()).collect())
				} else {
					Data::Dict(map)
				}
			}
			_ => Err(format!("unsupported value: {value:?}").into_lua_err())?,
		})
	}

	pub fn data_to_value(lua: &Lua, data: Data) -> mlua::Result<Value> {
		Ok(match data {
			Data::String(Cow::Owned(s)) => Value::String(lua.create_external_string(s)?),
			Data::List(l) => {
				let mut vec = Vec::with_capacity(l.len());
				for v in l.into_iter() {
					vec.push(Self::data_to_value(lua, v)?);
				}
				Value::Table(lua.create_sequence_from(vec)?)
			}
			Data::Dict(d) => {
				let seq_len = d.keys().filter(|&k| k.is_integer()).count();
				let tbl = lua.create_table_with_capacity(seq_len, d.len() - seq_len)?;
				for (k, v) in d {
					tbl.raw_set(Self::key_to_value(lua, k)?, Self::data_to_value(lua, v)?)?;
				}
				Value::Table(tbl)
			}
			Data::Url(u) => Value::String(lua.create_string(u.os_str().as_encoded_bytes())?),
			Data::Path(p) => Value::String(lua.create_string(p.encoded_bytes())?),
			Data::Bytes(b) => Value::String(lua.create_external_string(b)?),
			_ => Self::data_to_value_ref(lua, &data)?,
		})
	}

	pub fn data_to_value_ref(lua: &Lua, data: &Data) -> mlua::Result<Value> {
		Ok(match data {
			Data::Nil => Value::Nil,
			Data::Boolean(b) => Value::Boolean(*b),
			Data::Integer(i) => Value::Integer(*i),
			Data::Number(n) => Value::Number(*n),
			Data::String(s) => Value::String(lua.create_string(&**s)?),
			Data::List(l) => {
				let mut vec = Vec::with_capacity(l.len());
				for v in l {
					vec.push(Self::data_to_value_ref(lua, v)?);
				}
				Value::Table(lua.create_sequence_from(vec)?)
			}
			Data::Dict(d) => {
				let seq_len = d.keys().filter(|&k| k.is_integer()).count();
				let tbl = lua.create_table_with_capacity(seq_len, d.len() - seq_len)?;
				for (k, v) in d {
					tbl.raw_set(Self::key_to_value_ref(lua, k)?, Self::data_to_value_ref(lua, v)?)?;
				}
				Value::Table(tbl)
			}
			Data::Id(i) => Value::Integer(i.get() as i64),
			Data::Url(u) => Value::String(lua.create_string(u.os_str().as_encoded_bytes())?),
			Data::Path(p) => Value::String(lua.create_string(p.encoded_bytes())?),
			Data::Bytes(b) => Value::String(lua.create_string(b)?),
			Data::Any(_) => Err("unsupported DataAny in sendable".into_lua_err())?,
		})
	}

	fn value_to_key(value: Value) -> mlua::Result<DataKey> {
		Ok(match value {
			Value::Nil => DataKey::Nil,
			Value::Boolean(b) => DataKey::Boolean(b),
			Value::Integer(i) => DataKey::Integer(i),
			Value::Number(n) => DataKey::Number(ordered_float::OrderedFloat(n)),
			Value::String(s) => {
				if let Ok(s) = s.to_str() {
					DataKey::String(s.to_owned().into())
				} else {
					DataKey::Bytes(s.as_bytes().to_owned())
				}
			}
			_ => Err(format!("unsupported key: {value:?}").into_lua_err())?,
		})
	}

	fn key_to_value(lua: &Lua, key: DataKey) -> mlua::Result<Value> {
		match key {
			DataKey::String(Cow::Owned(s)) => lua.create_external_string(s).map(Value::String),
			DataKey::Bytes(b) => lua.create_external_string(b).map(Value::String),
			_ => Self::key_to_value_ref(lua, &key),
		}
	}

	fn key_to_value_ref(lua: &Lua, key: &DataKey) -> mlua::Result<Value> {
		Ok(match key {
			DataKey::Nil => Value::Nil,
			DataKey::Boolean(b) => Value::Boolean(*b),
			DataKey::Integer(i) => Value::Integer(*i),
			DataKey::Number(n) => Value::Number(n.0),
			DataKey::String(s) => Value::String(lua.create_string(&**s)?),
			DataKey::Id(i) => Value::Integer(i.get() as i64),
			DataKey::Url(u) => Value::String(lua.create_string(u.os_str().as_encoded_bytes())?),
			DataKey::Path(p) => Value::String(lua.create_string(p.encoded_bytes())?),
			DataKey::Bytes(b) => Value::String(lua.create_string(b)?),
		})
	}
}

impl mlua::IntoLua for Data {
	fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
		Sendable::data_to_value(lua, self)
	}
}

impl mlua::IntoLua for &Data {
	fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
		Sendable::data_to_value_ref(lua, self)
	}
}
