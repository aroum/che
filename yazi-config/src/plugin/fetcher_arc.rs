use std::{ops::Deref, sync::Arc};

use yazi_shared::event::Action;

use super::Fetcher;

#[derive(Clone, Debug)]
pub struct FetcherArc {
	pub inner: Arc<Fetcher>,
	pub idx:   u8,
	pub rev:   u16,
}

impl Deref for FetcherArc {
	type Target = Action;

	fn deref(&self) -> &Self::Target { &self.inner.run }
}

impl From<Fetcher> for FetcherArc {
	fn from(value: Fetcher) -> Self {
		let idx = value.idx;
		Self { inner: Arc::new(value), idx, rev: 0 }
	}
}

impl From<Arc<Fetcher>> for FetcherArc {
	fn from(inner: Arc<Fetcher>) -> Self {
		let idx = inner.idx;
		Self { inner, idx, rev: 0 }
	}
}
