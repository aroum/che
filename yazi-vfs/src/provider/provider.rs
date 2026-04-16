use std::io;

use tokio::sync::mpsc;
use yazi_fs::{cha::{Cha, ChaMode}, provider::{Attrs, Capabilities, Provider, local::Local}};
use yazi_shared::{path::PathBufDyn, strand::AsStrand, url::{AsUrl, Url, UrlBuf, UrlCow}};

use super::{Providers, ReadDir, RwFile};

pub async fn absolute<'a, U>(url: &'a U) -> io::Result<UrlCow<'a>>
where
	U: AsUrl,
{
	let u = url.as_url();
	if u.is_archive() {
		return Ok(u.into());
	}
	Providers::new(u).await?.absolute().await
}

pub async fn calculate<U>(url: U) -> io::Result<u64>
where
	U: AsUrl,
{
	let url = url.as_url();
	if let Some(path) = url.as_local() {
		yazi_fs::provider::local::SizeCalculator::total(path).await
	} else {
		super::SizeCalculator::total(url).await
	}
}

pub async fn canonicalize<U>(url: U) -> io::Result<UrlBuf>
where
	U: AsUrl,
{
	let u = url.as_url();
	if u.is_archive() {
		return Ok(u.to_owned());
	}
	Providers::new(u).await?.canonicalize().await
}

pub async fn capabilities<U>(url: U) -> io::Result<Capabilities>
where
	U: AsUrl,
{
	let u = url.as_url();
	if u.is_archive() {
		return Ok(Capabilities { symlink: false });
	}
	Ok(Providers::new(u).await?.capabilities())
}

pub async fn casefold<U>(url: U) -> io::Result<UrlBuf>
where
	U: AsUrl,
{
	let u = url.as_url();
	if u.is_archive() {
		return Ok(u.to_owned());
	}
	Providers::new(u).await?.casefold().await
}

pub async fn copy<U, V>(from: U, to: V, attrs: Attrs) -> io::Result<u64>
where
	U: AsUrl,
	V: AsUrl,
{
	let (from, to) = (from.as_url(), to.as_url());
	if to.is_archive() {
		return super::archive::copy(&from.to_owned(), &to.to_owned()).await;
	}

	let res = match (from.kind().is_local(), to.kind().is_local()) {
		(true, true) => Local::new(from).await?.copy(to.loc(), attrs).await,
		(false, false) if from.scheme().covariant(to.scheme()) => {
			Providers::new(from).await?.copy(to.loc(), attrs).await
		}
		(true, false) | (false, true) | (false, false) => super::copy_impl(from, to, attrs).await,
	};
	if res.is_ok() {
		if let (Some(f), Some(t)) = (from.loc().as_os().ok(), to.loc().as_os().ok()) {
			super::descr::copy_description(f, t);
		}
	}
	res
}

pub async fn copy_with_progress<U, V, A>(
	from: U,
	to: V,
	attrs: A,
) -> io::Result<mpsc::Receiver<Result<u64, io::Error>>>
where
	U: AsUrl,
	V: AsUrl,
	A: Into<Attrs>,
{
	let (from, to) = (from.as_url(), to.as_url());
	if to.is_archive() {
		let (from, to) = (from.to_owned(), to.to_owned());
		let (tx, rx) = mpsc::channel(10);
		tokio::spawn(async move {
			let res = super::archive::copy(&from, &to).await;
			match res {
				Ok(n) => {
					tx.send(Ok(n)).await.ok();
					tx.send(Ok(0)).await.ok();
				}
				Err(e) => {
					tx.send(Err(e)).await.ok();
				}
			}
		});
		return Ok(rx);
	}

	let res = match (from.kind().is_local(), to.kind().is_local()) {
		(true, true) => Local::new(from).await?.copy_with_progress(to.loc(), attrs),
		(false, false) if from.scheme().covariant(to.scheme()) => {
			Providers::new(from).await?.copy_with_progress(to.loc(), attrs)
		}
		(true, false) | (false, true) | (false, false) => {
			Ok(super::copy_with_progress_impl(from.to_owned(), to.to_owned(), attrs.into()))
		}
	};
	if res.is_ok() {
		if let (Some(f), Some(t)) = (from.loc().as_os().ok(), to.loc().as_os().ok()) {
			super::descr::copy_description(f, t);
		}
	}
	res
}

pub async fn create<U>(url: U) -> io::Result<RwFile>
where
	U: AsUrl,
{
	let url = url.as_url();
	if url.is_archive() {
		return Err(io::Error::new(io::ErrorKind::PermissionDenied, "Archive is read-only"));
	}
	Providers::new(url).await?.create().await
}

pub async fn create_dir<U>(url: U) -> io::Result<()>
where
	U: AsUrl,
{
	let url = url.as_url();
	if url.is_archive() { return Ok(()); }
	Providers::new(url).await?.create_dir().await
}

pub async fn create_dir_all<U>(url: U) -> io::Result<()>
where
	U: AsUrl,
{
	let url = url.as_url();
	if url.is_archive() { return Ok(()); }
	Providers::new(url).await?.create_dir_all().await
}

pub async fn create_new<U>(url: U) -> io::Result<RwFile>
where
	U: AsUrl,
{
	let url = url.as_url();
	if url.is_archive() {
		return Err(io::Error::new(io::ErrorKind::PermissionDenied, "Archive is read-only"));
	}
	Providers::new(url).await?.create_new().await
}

pub async fn hard_link<U, V>(original: U, link: V) -> io::Result<()>
where
	U: AsUrl,
	V: AsUrl,
{
	let (original, link) = (original.as_url(), link.as_url());
	if original.scheme().covariant(link.scheme()) {
		Providers::new(original).await?.hard_link(link.loc()).await
	} else {
		Err(io::Error::from(io::ErrorKind::CrossesDevices))
	}
}

pub async fn identical<U, V>(a: U, b: V) -> io::Result<bool>
where
	U: AsUrl,
	V: AsUrl,
{
	let (a_url, b_url) = (a.as_url(), b.as_url());
	if a_url.is_archive() || b_url.is_archive() {
		return Ok(a_url == b_url);
	}
	if let (Some(a), Some(b)) = (a_url.as_local(), b_url.as_local()) {
		yazi_fs::provider::local::identical(a, b).await
	} else {
		Ok(a_url == b_url)
	}
}

pub async fn metadata<U>(url: U) -> io::Result<Cha>
where
	U: AsUrl,
{
	use yazi_shared::url::UrlLike;
	use yazi_fs::provider::DirReader;
	use yazi_fs::provider::FileHolder;
	use yazi_shared::strand::StrandLike;

	let url = url.as_url();
	if url.is_archive() {
		let url_owned = url.to_owned();
		let is_dir = if let Url::Archive { loc, .. } = url {
			let (_, _, urn) = loc.triple();
			let urn_s = urn.to_string_lossy();
			// Empty urn = archive root = directory
			// Non-empty urn without trailing slash = look up actual type
			if urn_s.is_empty() {
				true
			} else if let Some(parent_url) = url_owned.parent().map(|u| u.to_owned()) {
				let target_name = urn_s.trim_end_matches('/').to_string();
				match super::archive::ReadDir::new(&parent_url).await {
					Ok(mut rd) => {
						let mut result = false;
						loop {
							match rd.next().await {
								Ok(Some(e)) => {
									let entry_name = e.name();
								let name_str = entry_name.to_string_lossy();
									if name_str == target_name {
										result = e.metadata().await.map(|c| c.is_dir()).unwrap_or(false);
										break;
									}
								}
								_ => break,
							}
						}
						result
					}
					// Fallback: treat as file if urn has extension, dir otherwise
					Err(_) => std::path::Path::new(urn_s.as_ref()).extension().is_none(),
				}
			} else {
				// No parent → archive root
				true
			}
		} else {
			true
		};
		let mode = if is_dir { yazi_fs::cha::ChaMode::T_DIR } else { yazi_fs::cha::ChaMode::T_FILE };
		return Ok(Cha { kind: yazi_fs::cha::ChaKind::empty(), mode, ..Default::default() });
	}
	Providers::new(url).await?.metadata().await
}

pub async fn must_identical<U, V>(a: U, b: V) -> bool
where
	U: AsUrl,
	V: AsUrl,
{
	identical(a, b).await.unwrap_or(false)
}

pub async fn open<U>(url: U) -> io::Result<RwFile>
where
	U: AsUrl,
{
	let url = url.as_url();
	if url.is_archive() {
		return super::archive::open(&url.to_owned()).await;
	}
	Providers::new(url).await?.open().await
}

pub async fn read_dir<U>(url: U) -> io::Result<ReadDir>
where
	U: AsUrl,
{
	let url = url.as_url();
	if url.is_archive() {
		return Ok(ReadDir::Archive(super::archive::ReadDir::new(&url.to_owned()).await?));
	}
	Providers::new(url).await?.read_dir().await
}

pub async fn read_link<U>(url: U) -> io::Result<PathBufDyn>
where
	U: AsUrl,
{
	let u = url.as_url();
	if u.is_archive() {
		return Err(io::Error::new(io::ErrorKind::NotFound, "Archive entries are not symlinks"));
	}
	Providers::new(u).await?.read_link().await
}

pub async fn remove_dir<U>(url: U) -> io::Result<()>
where
	U: AsUrl,
{
	let url = url.as_url();
	if url.is_archive() {
		return super::archive::remove_dir(&url.to_owned()).await;
	}
	Providers::new(url).await?.remove_dir().await
}

pub async fn remove_dir_all<U>(url: U) -> io::Result<()>
where
	U: AsUrl,
{
	let url = url.as_url();
	if url.is_archive() {
		return super::archive::remove_dir(&url.to_owned()).await;
	}
	Providers::new(url).await?.remove_dir_all().await
}

pub async fn remove_dir_clean<U>(url: U) -> io::Result<()>
where
	U: AsUrl,
{
	let url = url.as_url();
	if url.is_archive() {
		return super::archive::remove_dir(&url.to_owned()).await;
	}
	Providers::new(url).await?.remove_dir_clean().await
}

pub async fn remove_file<U>(url: U) -> io::Result<()>
where
	U: AsUrl,
{
	let url_ref = url.as_url();
	if url_ref.is_archive() {
		return super::archive::remove_file(&url_ref.to_owned()).await;
	}
	let res = Providers::new(url_ref).await?.remove_file().await;
	if res.is_ok() {
		if let Some(path) = url_ref.loc().as_os().ok() {
			super::descr::write_description(path, "");
		}
	}
	res
}

pub async fn rename<U, V>(from: U, to: V) -> io::Result<()>
where
	U: AsUrl,
	V: AsUrl,
{
	let (from, to) = (from.as_url(), to.as_url());
	if from.is_archive() && to.is_archive() {
		return super::archive::rename(&from.to_owned(), &to.to_owned()).await;
	} else if from.is_archive() || to.is_archive() {
		return Err(io::Error::new(io::ErrorKind::CrossesDevices, "Cannot rename between archive and local filesystem"));
	}
	let res = if from.scheme().covariant(to.scheme()) {
		Providers::new(from).await?.rename(to.loc()).await
	} else {
		Err(io::Error::from(io::ErrorKind::CrossesDevices))
	};
	if res.is_ok() {
		if let (Some(f), Some(t)) = (from.loc().as_os().ok(), to.loc().as_os().ok()) {
			super::descr::move_description(f, t);
		}
	}
	res
}

pub async fn set_mode<U>(url: U, mode: ChaMode) -> io::Result<()>
where
	U: AsUrl,
{
	Providers::new(url.as_url()).await?.set_mode(mode).await
}

pub async fn symlink<U, S, F>(link: U, original: S, is_dir: F) -> io::Result<()>
where
	U: AsUrl,
	S: AsStrand,
	F: AsyncFnOnce() -> io::Result<bool>,
{
	if link.as_url().is_archive() {
		return Err(io::Error::new(io::ErrorKind::PermissionDenied, "Archive is read-only"));
	}
	Providers::new(link.as_url()).await?.symlink(original, is_dir).await
}

pub async fn symlink_dir<U, S>(link: U, original: S) -> io::Result<()>
where
	U: AsUrl,
	S: AsStrand,
{
	if link.as_url().is_archive() {
		return Err(io::Error::new(io::ErrorKind::PermissionDenied, "Archive is read-only"));
	}
	Providers::new(link.as_url()).await?.symlink_dir(original).await
}

pub async fn symlink_file<U, S>(link: U, original: S) -> io::Result<()>
where
	U: AsUrl,
	S: AsStrand,
{
	if link.as_url().is_archive() {
		return Err(io::Error::new(io::ErrorKind::PermissionDenied, "Archive is read-only"));
	}
	Providers::new(link.as_url()).await?.symlink_file(original).await
}

pub async fn symlink_metadata<U>(url: U) -> io::Result<Cha>
where
	U: AsUrl,
{
	let url = url.as_url();
	if url.is_archive() { return metadata(url).await; }
	Providers::new(url).await?.symlink_metadata().await
}

pub async fn trash<U>(url: U) -> io::Result<UrlBuf>
where
	U: AsUrl,
{
	let url = url.as_url();
	if url.is_archive() {
		return Err(io::Error::new(io::ErrorKind::PermissionDenied, "Archive is read-only"));
	}
	Providers::new(url).await?.trash().await
}

pub fn try_absolute<'a, U>(url: U) -> Option<UrlCow<'a>>
where
	U: Into<UrlCow<'a>>,
{
	let url = url.into();
	match url.as_url() {
		Url::Regular(_) | Url::Search { .. } => yazi_fs::provider::local::try_absolute(url),
		Url::Archive { .. } => None, // TODO
		Url::Sftp { .. } => crate::provider::sftp::try_absolute(url),
	}
}

pub async fn write<U, C>(url: U, contents: C) -> io::Result<()>
where
	U: AsUrl,
	C: AsRef<[u8]>,
{
	Providers::new(url.as_url()).await?.write(contents).await
}
