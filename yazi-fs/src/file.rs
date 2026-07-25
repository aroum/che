use std::{hash::{Hash, Hasher}, ops::Deref, path::Path};

use yazi_shared::{path::{PathBufDyn, PathDyn}, strand::Strand, url::{UrlBuf, UrlLike}};

use crate::cha::{Cha, ChaType};

#[derive(Clone, Debug, Default)]
pub struct File {
	pub url:     UrlBuf,
	pub cha:     Cha,
	pub link_to: Option<PathBufDyn>,
	pub is_upparent: bool,
}

impl Deref for File {
	type Target = Cha;

	fn deref(&self) -> &Self::Target { &self.cha }
}

impl From<&Self> for File {
	fn from(value: &Self) -> Self { value.clone() }
}

impl File {
	#[inline]
	pub fn from_dummy(url: impl Into<UrlBuf>, r#type: Option<ChaType>) -> Self {
		let url = url.into();
		let cha = Cha::from_dummy(&url, r#type);
		Self { url, cha, link_to: None, is_upparent: false }
	}

	#[inline]
	pub fn make_upparent(parent_url: impl Into<UrlBuf>) -> Self {
		let url = parent_url.into();
		let mut cha = Cha::default();
		cha.mode = crate::cha::ChaMode::T_DIR;
		Self {
			url,
			cha,
			link_to: None,
			is_upparent: true,
		}
	}

	#[inline]
	pub fn chdir(&self, wd: &Path) -> Self {
		Self { url: self.url.rebase(wd), cha: self.cha, link_to: self.link_to.clone(), is_upparent: self.is_upparent }
	}
}

impl File {
	// --- Url
	#[inline]
	pub fn url_owned(&self) -> UrlBuf { self.url.clone() }

	#[inline]
	pub fn uri(&self) -> PathDyn<'_> { self.url.uri() }

	#[inline]
	pub fn urn(&self) -> PathDyn<'_> { self.url.urn() }

	#[inline]
	pub fn name(&self) -> Option<Strand<'_>> {
		if self.is_upparent {
			Some(Strand::Utf8(".."))
		} else {
			self.url.name()
		}
	}

	#[inline]
	pub fn stem(&self) -> Option<Strand<'_>> { self.url.stem() }
}

impl Hash for File {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.url.hash(state);
		self.cha.len.hash(state);
		self.cha.btime.hash(state);
		self.cha.ctime.hash(state);
		self.cha.mtime.hash(state);
	}
}
