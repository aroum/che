use std::{future, mem};

use mlua::{AnyUserData, FromLua, Lua, MetaMethod, UserData, UserDataMethods, Value};
use tokio_util::sync::CancellationToken;

#[derive(Clone, Debug, Default)]
pub struct Scope(Option<CancellationToken>);

impl Default for &Scope {
	fn default() -> Self {
		static SCOPE: Scope = Scope(None);
		&SCOPE
	}
}

impl From<&Scope> for Scope {
	fn from(value: &Self) -> Self { value.clone() }
}

impl From<CancellationToken> for Scope {
	fn from(value: CancellationToken) -> Self { Self(Some(value)) }
}

impl From<Option<CancellationToken>> for Scope {
	fn from(value: Option<CancellationToken>) -> Self { Self(value) }
}

impl Scope {
	pub fn new() -> Self { Self(Some(CancellationToken::new())) }

	pub fn child(&self) -> Self {
		Self(Some(self.0.as_ref().map_or_else(CancellationToken::new, CancellationToken::child_token)))
	}

	pub fn cancel(&self) {
		if let Some(token) = &self.0 {
			token.cancel();
		}
	}

	pub fn is_cancelled(&self) -> bool {
		self.0.as_ref().is_some_and(CancellationToken::is_cancelled)
	}

	pub async fn cancelled(&self) {
		if let Some(token) = &self.0 {
			token.cancelled().await;
		} else {
			future::pending::<()>().await;
		}
	}

	pub fn take(&mut self) -> Self { mem::take(self) }
}

impl FromLua for Scope {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		if value.is_nil() {
			Ok(Self::default())
		} else {
			AnyUserData::from_lua(value, lua)?.borrow::<Self>().map(|ud| ud.clone())
		}
	}
}

impl UserData for Scope {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_method("cancel", |_, me, ()| Ok(me.cancel()));
		methods.add_method("cancelled", |_, me, ()| Ok(me.is_cancelled()));
		methods.add_method("child", |_, me, ()| Ok(me.child()));

		methods
			.add_async_meta_method(MetaMethod::Call, |_, me, ()| async move { Ok(me.cancelled().await) });
	}
}
