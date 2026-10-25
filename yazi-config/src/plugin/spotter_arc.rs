use std::{ops::Deref, sync::Arc};

use yazi_shared::event::Action;

use super::Spotter;

#[derive(Clone, Debug)]
pub struct SpotterArc {
	pub inner: Arc<Spotter>,
}

impl Deref for SpotterArc {
	type Target = Action;

	fn deref(&self) -> &Self::Target { &self.inner.run }
}

impl From<Spotter> for SpotterArc {
	fn from(value: Spotter) -> Self { Self { inner: Arc::new(value) } }
}

impl From<Arc<Spotter>> for SpotterArc {
	fn from(inner: Arc<Spotter>) -> Self { Self { inner } }
}
