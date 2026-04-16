use std::{io, path::Path, sync::Arc};

use tokio::sync::mpsc;
use yazi_shared::{path::{AsPath, PathBufDyn}, scheme::SchemeKind, strand::AsStrand, url::{Url, UrlBuf, UrlCow}};

use crate::{cha::{Cha, ChaMode}, provider::{Attrs, Capabilities, Provider}};

#[derive(Clone)]
pub struct Local<'a> {
	url:  Url<'a>,
	path: &'a Path,
}

impl<'a> Provider for Local<'a> {
	type File = tokio::fs::File;
	type Gate = super::Gate;
	type Me<'b> = Local<'b>;
	type ReadDir = super::ReadDir;
	type UrlCow = UrlCow<'a>;

	async fn absolute(&self) -> io::Result<Self::UrlCow> {
		super::try_absolute(self.url)
			.ok_or_else(|| io::Error::other("Cannot get absolute path for local URL"))
	}

	#[inline]
	async fn canonicalize(&self) -> io::Result<UrlBuf> {
		tokio::fs::canonicalize(self.path).await.map(Into::into)
	}

	#[inline]
	fn capabilities(&self) -> Capabilities { Capabilities { symlink: true } }

	async fn casefold(&self) -> io::Result<UrlBuf> {
		super::casefold(self.path).await.map(Into::into)
	}

	#[inline]
	async fn copy<P>(&self, to: P, attrs: Attrs) -> io::Result<u64>
	where
		P: AsPath,
	{
		let to = to.as_path().to_os_owned()?;
		let from = self.path.to_owned();
		super::copy_impl(from, to, attrs).await
	}

	fn copy_with_progress<P, A>(&self, to: P, attrs: A) -> io::Result<mpsc::Receiver<io::Result<u64>>>
	where
		P: AsPath,
		A: Into<Attrs>,
	{
		let to = to.as_path().to_os_owned()?;
		let from = self.path.to_owned();
		Ok(super::copy_with_progress_impl(from, to, attrs.into()))
	}

	#[inline]
	async fn create_dir(&self) -> io::Result<()> { tokio::fs::create_dir(self.path).await }

	#[inline]
	async fn create_dir_all(&self) -> io::Result<()> { tokio::fs::create_dir_all(self.path).await }

	#[inline]
	async fn hard_link<P>(&self, to: P) -> io::Result<()>
	where
		P: AsPath,
	{
		let to = to.as_path().as_os()?;

		tokio::fs::hard_link(self.path, to).await
	}

	#[inline]
	async fn metadata(&self) -> io::Result<Cha> {
		Ok(Cha::new(self.path.file_name().unwrap_or_default(), tokio::fs::metadata(self.path).await?))
	}

	#[inline]
	async fn new<'b>(url: Url<'b>) -> io::Result<Self::Me<'b>> {
		match url {
			Url::Regular(loc) | Url::Search { loc, .. } => Ok(Self::Me { url, path: loc.as_inner() }),
			Url::Archive { .. } | Url::Sftp { .. } => {
				Err(io::Error::new(io::ErrorKind::InvalidInput, format!("Not a local URL: {url:?}")))
			}
		}
	}

	#[inline]
	async fn read_dir(self) -> io::Result<Self::ReadDir> {
		Ok(match self.url.kind() {
			SchemeKind::Regular => Self::ReadDir::Regular(tokio::fs::read_dir(self.path).await?),
			SchemeKind::Search => Self::ReadDir::Others {
				reader: tokio::fs::read_dir(self.path).await?,
				dir:    Arc::new(self.url.to_owned()),
			},
			SchemeKind::Archive | SchemeKind::Sftp => Err(io::Error::new(
				io::ErrorKind::InvalidInput,
				format!("Not a local URL: {:?}", self.url),
			))?,
		})
	}

	#[inline]
	async fn read_link(&self) -> io::Result<PathBufDyn> {
		Ok(tokio::fs::read_link(self.path).await?.into())
	}

	#[inline]
	async fn remove_dir(&self) -> io::Result<()> { tokio::fs::remove_dir(self.path).await }

	#[inline]
	async fn remove_dir_all(&self) -> io::Result<()> { tokio::fs::remove_dir_all(self.path).await }

	#[inline]
	async fn remove_file(&self) -> io::Result<()> { tokio::fs::remove_file(self.path).await }

	#[inline]
	async fn rename<P>(&self, to: P) -> io::Result<()>
	where
		P: AsPath,
	{
		let to = to.as_path().as_os()?;

		tokio::fs::rename(self.path, to).await
	}

	async fn set_mode(&self, mode: ChaMode) -> io::Result<()> {
		#[cfg(unix)]
		{
			return tokio::fs::set_permissions(self.path, mode.into()).await;
		}
		#[cfg(windows)]
		{
			use std::os::windows::ffi::OsStrExt;

			let path: Vec<u16> = self.path.as_os_str().encode_wide().chain(Some(0)).collect();
			let perm = if mode.contains(ChaMode::U_WRITE) { libc::S_IWRITE } else { libc::S_IREAD };

			return tokio::task::spawn_blocking(move || {
				let result = unsafe { libc::wchmod(path.as_ptr(), perm) };
				if result == 0 { Ok(()) } else { Err(io::Error::last_os_error()) }
			})
			.await?;
		}
	}

	#[inline]
	async fn symlink<S, F>(&self, original: S, _is_dir: F) -> io::Result<()>
	where
		S: AsStrand,
		F: AsyncFnOnce() -> io::Result<bool>,
	{
		#[cfg(unix)]
		{
			let original = original.as_strand().as_os()?;
			tokio::fs::symlink(original, self.path).await
		}
		#[cfg(windows)]
		if _is_dir().await? {
			self.symlink_dir(original).await
		} else {
			self.symlink_file(original).await
		}
	}

	#[inline]
	async fn symlink_dir<S>(&self, original: S) -> io::Result<()>
	where
		S: AsStrand,
	{
		let original = original.as_strand().as_os()?;

		#[cfg(unix)]
		{
			tokio::fs::symlink(original, self.path).await
		}
		#[cfg(windows)]
		{
			tokio::fs::symlink_dir(original, self.path).await
		}
	}

	#[inline]
	async fn symlink_file<S>(&self, original: S) -> io::Result<()>
	where
		S: AsStrand,
	{
		let original = original.as_strand().as_os()?;

		#[cfg(unix)]
		{
			tokio::fs::symlink(original, self.path).await
		}
		#[cfg(windows)]
		{
			tokio::fs::symlink_file(original, self.path).await
		}
	}

	#[inline]
	async fn symlink_metadata(&self) -> io::Result<Cha> {
		Ok(Cha::new(
			self.path.file_name().unwrap_or_default(),
			tokio::fs::symlink_metadata(self.path).await?,
		))
	}

	async fn trash(&self) -> io::Result<UrlBuf> {
		let path = self.path.to_owned();
		tokio::task::spawn_blocking(move || {
			#[cfg(target_os = "android")]
			{
				Err(io::Error::new(io::ErrorKind::Unsupported, "Unsupported OS for trash operation"))
			}
			#[cfg(target_os = "macos")]
			{
				use std::ffi::{CStr, CString, c_char};
				use std::os::unix::ffi::OsStrExt;
				use std::path::PathBuf;
				use objc2::msg_send;
				use objc2::runtime::{AnyClass, AnyObject};

				let path_cstr = CString::new(path.as_os_str().as_bytes())
					.map_err(|e| io::Error::other(format!("Invalid path: {e}")))?;

				unsafe {
					let cls_str = AnyClass::get(c"NSString").unwrap();
					let cls_url = AnyClass::get(c"NSURL").unwrap();
					let cls_fm = AnyClass::get(c"NSFileManager").unwrap();

					#[allow(unexpected_cfgs)]
					let ns_path: *const AnyObject =
						msg_send![cls_str, stringWithUTF8String: path_cstr.as_ptr()];
					#[allow(unexpected_cfgs)]
					let ns_url: *const AnyObject = msg_send![cls_url, fileURLWithPath: ns_path];
					#[allow(unexpected_cfgs)]
					let fm: *const AnyObject = msg_send![cls_fm, defaultManager];

					let mut result_url: *const AnyObject = std::ptr::null();
					let mut error: *const AnyObject = std::ptr::null();
					#[allow(unexpected_cfgs)]
					let success: bool = msg_send![
						fm,
						trashItemAtURL: ns_url,
						resultingItemURL: &mut result_url,
						error: &mut error
					];

					if !success {
						if !error.is_null() {
							#[allow(unexpected_cfgs)]
							let desc: *const AnyObject = msg_send![error, localizedDescription];
							#[allow(unexpected_cfgs)]
							let cstr: *const c_char = msg_send![desc, UTF8String];
							let err_msg = CStr::from_ptr(cstr).to_string_lossy();
							return Err(io::Error::other(format!("Trash failed: {err_msg}")));
						}
						return Err(io::Error::other("Trash operation failed"));
					}

					if result_url.is_null() {
						return Err(io::Error::other("Trash succeeded but no resulting URL"));
					}

					#[allow(unexpected_cfgs)]
					let ns_result_path: *const AnyObject = msg_send![result_url, path];
					if ns_result_path.is_null() {
						return Err(io::Error::other("Cannot get path from trash URL"));
					}
					#[allow(unexpected_cfgs)]
					let cstr: *const c_char = msg_send![ns_result_path, UTF8String];
					let trash_path = PathBuf::from(std::ffi::OsStr::from_bytes(
						CStr::from_ptr(cstr).to_bytes(),
					));

					Ok(UrlBuf::from(trash_path))
				}
			}
			#[cfg(all(not(target_os = "macos"), not(target_os = "android")))]
			{
				use std::collections::HashSet;

				let home_trash = dirs::data_dir()
					.or_else(|| dirs::home_dir().map(|h| h.join(".local/share")))
					.map(|d| d.join("Trash"))
					.ok_or_else(|| io::Error::other("Cannot determine trash directory"))?;

				let info_dir = home_trash.join("info");
				let files_dir = home_trash.join("files");

				// Snapshot existing trashinfo entries before deletion
				let before: HashSet<std::ffi::OsString> = std::fs::read_dir(&info_dir)
					.ok()
					.into_iter()
					.flatten()
					.filter_map(|e| e.ok())
					.map(|e| e.file_name())
					.collect();

				trash::delete(&path).map_err(io::Error::other)?;

				// Find the new trashinfo entry
				if let Ok(entries) = std::fs::read_dir(&info_dir) {
					for entry in entries.filter_map(|e| e.ok()) {
						let name = entry.file_name();
						if !before.contains(&name) {
							if let Some(stem) = name.to_string_lossy().strip_suffix(".trashinfo") {
								return Ok(UrlBuf::from(files_dir.join(stem)));
							}
						}
					}
				}

				// Fallback: construct from filename
				let filename = path
					.file_name()
					.ok_or_else(|| io::Error::other("Cannot determine filename"))?;
				Ok(UrlBuf::from(files_dir.join(filename)))
			}
		})
		.await?
	}

	#[inline]
	fn url(&self) -> Url<'_> { self.url }

	#[inline]
	async fn write<C>(&self, contents: C) -> io::Result<()>
	where
		C: AsRef<[u8]>,
	{
		tokio::fs::write(self.path, contents).await
	}
}

impl<'a> Local<'a> {
	#[inline]
	pub async fn read(&self) -> io::Result<Vec<u8>> { tokio::fs::read(self.path).await }

	#[inline]
	pub async fn read_to_string(&self) -> io::Result<String> {
		tokio::fs::read_to_string(self.path).await
	}

	#[inline]
	pub fn regular<P>(path: &'a P) -> Self
	where
		P: ?Sized + AsRef<Path>,
	{
		Self { url: Url::regular(path), path: path.as_ref() }
	}
}
