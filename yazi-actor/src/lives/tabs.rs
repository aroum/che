use std::ops::Deref;

use mlua::{AnyUserData, MetaMethod, UserData, UserDataFields, UserDataMethods};

use super::{Lives, Pane, PtrCell, Tab};

pub(super) struct Tabs {
	inner: PtrCell<yazi_core::mgr::Tabs>,
}

impl Deref for Tabs {
	type Target = yazi_core::mgr::Tabs;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl Tabs {
	pub(super) fn make(inner: &yazi_core::mgr::Tabs) -> mlua::Result<AnyUserData> {
		Lives::scoped_userdata(Self { inner: inner.into() })
	}
}

impl UserData for Tabs {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("idx", |_, me| Ok(me.active_pane + 1));
		fields.add_field_method_get("single_pane", |_, me| Ok(me.single_pane));
		fields.add_field_method_get("preview_pane", |_, me| Ok(me.preview_pane));
	}

	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method(MetaMethod::Len, |_, me, ()| Ok(me.panes.len()));

		methods.add_meta_method(MetaMethod::Index, |_, me, idx: usize| {
			if idx == 0 || idx > me.panes.len() {
				Ok(None)
			} else {
				Some(Tab::make(me.panes[idx - 1].active())).transpose()
			}
		});

		methods.add_method("pane", |_, me, idx: usize| {
			if idx == 0 || idx > me.panes.len() {
				Ok(None)
			} else {
				Some(Pane::make(&me.panes[idx - 1])).transpose()
			}
		});
	}
}
