use std::io;

use yazi_fs::engine::{Attrs, FileBuilder};
use yazi_shared::{scheme::SchemeKind, url::AsUrl};

#[derive(Clone, Copy, Default)]
pub struct Demand {
	pub(super) append:     bool,
	pub(super) attrs:      Attrs,
	pub(super) create:     bool,
	pub(super) create_new: bool,
	pub(super) read:       bool,
	pub(super) truncate:   bool,
	pub(super) write:      bool,
}

impl FileBuilder for Demand {
	type File = super::RwFile;

	fn append(&mut self, append: bool) -> &mut Self {
		self.append = append;
		self
	}

	fn attrs(&mut self, attrs: Attrs) -> &mut Self {
		self.attrs = attrs;
		self
	}

	fn create(&mut self, create: bool) -> &mut Self {
		self.create = create;
		self
	}

	fn create_new(&mut self, create_new: bool) -> &mut Self {
		self.create_new = create_new;
		self
	}

	async fn open<U>(&self, url: U) -> io::Result<Self::File>
	where
		U: AsUrl,
	{
		let url = url.as_url();
		Ok(match url.kind() {
			SchemeKind::Regular | SchemeKind::Search => {
				self.build::<yazi_fs::engine::local::Demand>().open(url).await?.into()
			}
			SchemeKind::Archive => {
				Err(io::Error::new(io::ErrorKind::Unsupported, "Unsupported filesystem: archive"))?
			}
			SchemeKind::Sftp => self.build::<super::sftp::Demand>().open(url).await?.into(),
		})
	}

	fn read(&mut self, read: bool) -> &mut Self {
		self.read = read;
		self
	}

	fn truncate(&mut self, truncate: bool) -> &mut Self {
		self.truncate = truncate;
		self
	}

	fn write(&mut self, write: bool) -> &mut Self {
		self.write = write;
		self
	}
}

impl Demand {
	fn build<T: FileBuilder>(self) -> T {
		let mut demand = T::default();
		if self.append {
			demand.append(true);
		}
		demand.attrs(self.attrs);
		if self.create {
			demand.create(true);
		}
		if self.create_new {
			demand.create_new(true);
		}
		if self.read {
			demand.read(true);
		}
		if self.truncate {
			demand.truncate(true);
		}
		if self.write {
			demand.write(true);
		}
		demand
	}
}

#[allow(dead_code)]
pub type Gate = Demand;
