use std::{ops::Deref, sync::Arc};

use yazi_shared::event::Action;

use super::Preloader;

#[derive(Clone, Debug)]
pub struct PreloaderArc {
	pub inner: Arc<Preloader>,
	pub idx:   u8,
	pub rev:   u16,
}

impl Deref for PreloaderArc {
	type Target = Action;

	fn deref(&self) -> &Self::Target { &self.inner.run }
}

impl From<Preloader> for PreloaderArc {
	fn from(value: Preloader) -> Self {
		let idx = value.idx;
		Self { inner: Arc::new(value), idx, rev: 0 }
	}
}

impl From<Arc<Preloader>> for PreloaderArc {
	fn from(inner: Arc<Preloader>) -> Self {
		let idx = inner.idx;
		Self { inner, idx, rev: 0 }
	}
}
