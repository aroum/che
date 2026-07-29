use std::{io, pin::Pin};

use tokio::io::{AsyncRead, AsyncSeek, AsyncWrite};
use yazi_fs::provider::Attrs;

pub enum RwFile {
	Tokio(tokio::fs::File),
	Sftp(Box<yazi_sftp::fs::File>),
	Archive(std::io::Cursor<Vec<u8>>),
}

impl From<tokio::fs::File> for RwFile {
	fn from(f: tokio::fs::File) -> Self { Self::Tokio(f) }
}

impl From<yazi_sftp::fs::File> for RwFile {
	fn from(f: yazi_sftp::fs::File) -> Self { Self::Sftp(Box::new(f)) }
}

impl RwFile {
	// FIXME: path
	pub async fn metadata(&self) -> io::Result<yazi_fs::cha::Cha> {
		Ok(match self {
			Self::Tokio(f) => yazi_fs::cha::Cha::new("// FIXME", f.metadata().await?),
			Self::Sftp(f) => super::sftp::Cha::try_from(("// FIXME".as_bytes(), &f.fstat().await?))?.0,
			Self::Archive(c) => yazi_fs::cha::Cha {
				kind: yazi_fs::cha::ChaKind::empty(),
				mode: yazi_fs::cha::ChaMode::T_FILE,
				len: c.get_ref().len() as u64,
				..Default::default()
			},
		})
	}

	pub async fn set_attrs(&self, attrs: Attrs) -> io::Result<()> {
		match self {
			Self::Tokio(f) => {
				let (perm, times) = (attrs.try_into(), attrs.try_into());
				if perm.is_err() && times.is_err() {
					return Ok(());
				}

				let std = f.try_clone().await?.into_std().await;
				tokio::task::spawn_blocking(move || {
					perm.map(|p| std.set_permissions(p)).ok();
					times.map(|t| std.set_times(t)).ok();
				})
				.await?;
			}
			Self::Sftp(f) => {
				if let Ok(attrs) = super::sftp::Attrs(attrs).try_into() {
					f.fsetstat(&attrs).await?;
				}
			}
			Self::Archive(_) => {
				return Err(io::Error::new(io::ErrorKind::PermissionDenied, "Archive is read-only"));
			}
		}

		Ok(())
	}

	pub async fn set_len(&self, size: u64) -> io::Result<()> {
		Ok(match self {
			Self::Tokio(f) => f.set_len(size).await?,
			Self::Sftp(f) => {
				f.fsetstat(&yazi_sftp::fs::Attrs { size: Some(size), ..Default::default() }).await?
			}
			Self::Archive(_) => {
				return Err(io::Error::new(io::ErrorKind::PermissionDenied, "Archive is read-only"));
			}
		})
	}
}

impl AsyncRead for RwFile {
	#[inline]
	fn poll_read(
		mut self: Pin<&mut Self>,
		cx: &mut std::task::Context<'_>,
		buf: &mut tokio::io::ReadBuf<'_>,
	) -> std::task::Poll<io::Result<()>> {
		match &mut *self {
			Self::Tokio(f) => Pin::new(f).poll_read(cx, buf),
			Self::Sftp(f) => Pin::new(f).poll_read(cx, buf),
			Self::Archive(c) => Pin::new(c).poll_read(cx, buf),
		}
	}
}

impl AsyncSeek for RwFile {
	#[inline]
	fn start_seek(mut self: Pin<&mut Self>, position: io::SeekFrom) -> io::Result<()> {
		match &mut *self {
			Self::Tokio(f) => Pin::new(f).start_seek(position),
			Self::Sftp(f) => Pin::new(f).start_seek(position),
			Self::Archive(c) => Pin::new(c).start_seek(position),
		}
	}

	#[inline]
	fn poll_complete(
		mut self: Pin<&mut Self>,
		cx: &mut std::task::Context<'_>,
	) -> std::task::Poll<io::Result<u64>> {
		match &mut *self {
			Self::Tokio(f) => Pin::new(f).poll_complete(cx),
			Self::Sftp(f) => Pin::new(f).poll_complete(cx),
			Self::Archive(c) => Pin::new(c).poll_complete(cx),
		}
	}
}

impl AsyncWrite for RwFile {
	#[inline]
	fn poll_write(
		mut self: Pin<&mut Self>,
		_cx: &mut std::task::Context<'_>,
		_buf: &[u8],
	) -> std::task::Poll<Result<usize, io::Error>> {
		match &mut *self {
			Self::Tokio(f) => Pin::new(f).poll_write(_cx, _buf),
			Self::Sftp(f) => Pin::new(f).poll_write(_cx, _buf),
			Self::Archive(_) => std::task::Poll::Ready(Err(io::Error::new(
				io::ErrorKind::PermissionDenied,
				"Archive is read-only",
			))),
		}
	}

	#[inline]
	fn poll_flush(
		mut self: Pin<&mut Self>,
		_cx: &mut std::task::Context<'_>,
	) -> std::task::Poll<Result<(), io::Error>> {
		match &mut *self {
			Self::Tokio(f) => Pin::new(f).poll_flush(_cx),
			Self::Sftp(f) => Pin::new(f).poll_flush(_cx),
			Self::Archive(_) => std::task::Poll::Ready(Ok(())),
		}
	}

	#[inline]
	fn poll_shutdown(
		mut self: Pin<&mut Self>,
		_cx: &mut std::task::Context<'_>,
	) -> std::task::Poll<Result<(), io::Error>> {
		match &mut *self {
			Self::Tokio(f) => Pin::new(f).poll_shutdown(_cx),
			Self::Sftp(f) => Pin::new(f).poll_shutdown(_cx),
			Self::Archive(_) => std::task::Poll::Ready(Ok(())),
		}
	}

	#[inline]
	fn poll_write_vectored(
		mut self: Pin<&mut Self>,
		_cx: &mut std::task::Context<'_>,
		bufs: &[io::IoSlice<'_>],
	) -> std::task::Poll<Result<usize, io::Error>> {
		match &mut *self {
			Self::Tokio(f) => Pin::new(f).poll_write_vectored(_cx, bufs),
			Self::Sftp(f) => Pin::new(f).poll_write_vectored(_cx, bufs),
			Self::Archive(_) => std::task::Poll::Ready(Err(io::Error::new(
				io::ErrorKind::PermissionDenied,
				"Archive is read-only",
			))),
		}
	}

	#[inline]
	fn is_write_vectored(&self) -> bool {
		match self {
			Self::Tokio(f) => f.is_write_vectored(),
			Self::Sftp(f) => f.is_write_vectored(),
			Self::Archive(_) => false,
		}
	}
}
