use mlua::{ExternalResult, FromLua, IntoLua, Lua, MultiValue, ObjectLike, Value};
use tokio::runtime::Handle;
use yazi_binding::{Error, File};
use yazi_dds::Sendable;
use yazi_shared::event::ActionCow;

use super::slim_lua;
use crate::loader::LOADER;

pub async fn fetch(
	action: ActionCow,
	files: Vec<yazi_fs::File>,
) -> mlua::Result<(FetchState, Option<Error>)> {
	if files.is_empty() {
		return Ok((FetchState::Bool(true), None));
	}
	LOADER.ensure(&action.name, |_| ()).await.into_lua_err()?;

	tokio::task::spawn_blocking(move || {
		let lua = slim_lua(&action.name)?;
		let job = lua.create_table_from([
			("args", Sendable::args_to_table_ref(&lua, &action.args)?.into_lua(&lua)?),
			("files", lua.create_sequence_from(files.into_iter().map(File::new))?.into_lua(&lua)?),
		])?;

		Handle::current().block_on(async {
			let plugin = LOADER.load(&lua, &action.name).await?;
			let mut values: MultiValue = plugin.call_async_method("fetch", job).await?;
			let first = values.pop_front().unwrap_or(Value::Nil);

			match first {
				Value::Function(func) => {
					// Modern fetcher returning coroutine generator from ya.co
					let mut err = None;
					let mut next_values: MultiValue = match func.call_async(()).await {
						Ok(v) => v,
						Err(e) => {
							return Ok((FetchState::Bool(false), Some(Error::custom(e.to_string()))));
						}
					};
					while !next_values.front().is_none_or(Value::is_nil) {
						match func.call_async(true).await {
							Ok(v) => next_values = v,
							Err(e) => {
								err = Some(Error::custom(e.to_string()));
								break;
							}
						}
					}
					Ok((FetchState::Bool(err.is_none()), err))
				}
				Value::Boolean(b) => {
					let err = values.pop_front().and_then(|v| Error::from_lua(v, &lua).ok());
					Ok((FetchState::Bool(b), err))
				}
				Value::Table(tbl) => {
					let v = tbl.sequence_values().collect::<mlua::Result<Vec<bool>>>()?;
					let err = values.pop_front().and_then(|v| Error::from_lua(v, &lua).ok());
					Ok((FetchState::Vec(v), err))
				}
				_ => Ok((FetchState::Bool(true), None)),
			}
		})
	})
	.await
	.into_lua_err()?
}

// --- State
pub enum FetchState {
	Bool(bool),
	Vec(Vec<bool>),
}

impl FetchState {
	#[inline]
	pub fn get(&self, idx: usize) -> bool {
		match self {
			Self::Bool(b) => *b,
			Self::Vec(v) => v.get(idx).copied().unwrap_or(false),
		}
	}
}

impl FromLua for FetchState {
	fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
		Ok(match value {
			Value::Boolean(b) => Self::Bool(b),
			Value::Table(tbl) => Self::Vec(tbl.sequence_values().collect::<mlua::Result<_>>()?),
			Value::Function(_) | Value::Nil => Self::Bool(true),
			_ => Self::Bool(true),
		})
	}
}
