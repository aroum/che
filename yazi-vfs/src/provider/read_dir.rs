use std::io;

use yazi_fs::provider::DirReader;

pub enum ReadDir {
	Local(yazi_fs::provider::local::ReadDir),
	Archive(super::archive::ReadDir),
	Sftp(super::sftp::ReadDir),
}

impl DirReader for ReadDir {
	type Entry = super::DirEntry;

	async fn next(&mut self) -> io::Result<Option<Self::Entry>> {
		Ok(match self {
			Self::Local(reader) => reader.next().await?.map(Self::Entry::Local),
			Self::Archive(reader) => reader.next().await?.map(Self::Entry::Archive),
			Self::Sftp(reader) => reader.next().await?.map(Self::Entry::Sftp),
		})
	}
}
