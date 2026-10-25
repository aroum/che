use std::{ops::Deref, sync::Arc};

use yazi_shared::event::Action;

use super::Previewer;

#[derive(Clone, Debug)]
pub struct PreviewerArc {
	pub inner: Arc<Previewer>,
}

impl Deref for PreviewerArc {
	type Target = Action;

	fn deref(&self) -> &Self::Target { &self.inner.run }
}

impl From<Previewer> for PreviewerArc {
	fn from(value: Previewer) -> Self { Self { inner: Arc::new(value) } }
}

impl From<Arc<Previewer>> for PreviewerArc {
	fn from(inner: Arc<Previewer>) -> Self { Self { inner } }
}
