use std::borrow::Cow;

use yazi_shared::{loc::LocBuf, path::{PathBufDyn, PathCow, PathKind, PathLike}, pool::InternStr, url::{AsUrl, Url, UrlBuf, UrlCow, UrlLike}, wtf8::FromWtf8Vec};

#[inline]
pub fn expand_url<'a>(url: impl Into<UrlCow<'a>>) -> UrlCow<'a> { expand_url_impl(url.into()) }

fn expand_url_impl(url: UrlCow) -> UrlCow {
	let (base, rest, urn) = url.triple();

	let base = expand_variables(base.into());
	let rest = expand_variables(rest.into());
	let urn = expand_variables(urn.into());
	if base.is_borrowed() && rest.is_borrowed() && urn.is_borrowed() {
		return url;
	}

	let mut path = PathBufDyn::with_capacity(url.kind(), base.len() + rest.len() + urn.len());
	path.try_push(&base).expect("push original base should not fail");
	let c_base = path.components().count();

	path.try_push(&rest).expect("push original URI should not fail");
	let c_trail = path.components().count();

	path.try_push(&urn).expect("push original URN should not fail");
	let c_full = path.components().count();

	let uri = if urn.has_prefix() || rest.has_prefix() {
		c_full
	} else if urn.has_root() || rest.has_root() {
		c_full - c_base.min(path.has_prefix() as usize)
	} else {
		c_full - c_base
	};
	let urn = if urn.has_prefix() || urn.has_root() {
		path.components().rev().take_while(|&c| c != yazi_shared::path::Component::RootDir).count()
	} else {
		c_full - c_trail
	};

	match url.as_url() {
		Url::Regular(_) => UrlBuf::from(path.into_os().unwrap()),
		Url::Search { domain, .. } => UrlBuf::Search {
			loc:    LocBuf::<std::path::PathBuf>::with(path.into_os().unwrap(), uri, urn).unwrap(),
			domain: domain.intern(),
		},
		Url::Archive { domain, .. } => UrlBuf::Archive {
			loc:    LocBuf::<std::path::PathBuf>::with(path.into_os().unwrap(), uri, urn).unwrap(),
			domain: domain.intern(),
		},
		Url::Sftp { domain, .. } => UrlBuf::Sftp {
			loc:    LocBuf::<typed_path::UnixPathBuf>::with(path.into_unix().unwrap(), uri, urn).unwrap(),
			domain: domain.intern(),
		},
	}
	.into()
}

fn expand_variables(p: PathCow) -> PathCow {
	// ${HOME} or $HOME
	#[cfg(unix)]
	let re = regex::bytes::Regex::new(r"\$(?:\{([^}]+)\}|([a-zA-Z\d_]+))").unwrap();

	// %USERPROFILE%
	#[cfg(windows)]
	let re = regex::bytes::Regex::new(r"%([^%]+)%").unwrap();

	let b = p.encoded_bytes();
	let b = re.replace_all(b, |caps: &regex::bytes::Captures| {
		let name = caps.get(2).or_else(|| caps.get(1)).unwrap();
		str::from_utf8(name.as_bytes())
			.ok()
			.and_then(std::env::var_os)
			.map_or_else(|| caps.get(0).unwrap().as_bytes().to_owned(), |s| s.into_encoded_bytes())
	});

	match (b, p.kind()) {
		(Cow::Borrowed(_), _) => p,
		(Cow::Owned(b), PathKind::Os) => {
			PathBufDyn::Os(std::path::PathBuf::from_wtf8_vec(b).expect("valid WTF-8 path")).into()
		}
		(Cow::Owned(b), PathKind::Unix) => PathBufDyn::Unix(b.into()).into(),
	}
}

#[cfg(test)]
mod tests {
	use anyhow::Result;

	use super::*;

	#[cfg(unix)]
	#[test]
	fn test_expand_url() -> Result<()> {
		yazi_shared::init_tests();
		unsafe {
			std::env::set_var("FOO", "foo");
			std::env::set_var("BAR_BAZ", "bar/baz");
			std::env::set_var("BAR/BAZ", "bar_baz");
			std::env::set_var("EM/PT/Y", "");
		}

		let cases = [
			// Zero extra component expanded
			("archive:////tmp/test.zip/$FOO/bar", "archive:////tmp/test.zip/foo/bar"),
			("archive://:1//tmp/test.zip/$FOO/bar", "archive://:1//tmp/test.zip/foo/bar"),
			("archive://:2//tmp/test.zip/bar/$FOO", "archive://:2//tmp/test.zip/bar/foo"),
			("archive://:3//tmp/test.zip/$FOO/bar", "archive://:3//tmp/test.zip/foo/bar"),
			("archive://:3:1//tmp/test.zip/bar/$FOO", "archive://:3:1//tmp/test.zip/bar/foo"),
			("archive://:3:2//tmp/test.zip/$FOO/bar", "archive://:3:2//tmp/test.zip/foo/bar"),
			("archive://:3:3//tmp/test.zip/bar/$FOO", "archive://:3:3//tmp/test.zip/bar/foo"),
			// +1 component
			("archive:////tmp/test.zip/$BAR_BAZ", "archive:////tmp/test.zip/bar/baz"),
			("archive://:1//tmp/test.zip/$BAR_BAZ", "archive://:2//tmp/test.zip/bar/baz"),
			("archive://:2//$BAR_BAZ/tmp/test.zip", "archive://:2//bar/baz/tmp/test.zip"),
			("archive://:2:1//tmp/test.zip/$BAR_BAZ", "archive://:3:2//tmp/test.zip/bar/baz"),
			("archive://:2:2//tmp/$BAR_BAZ/test.zip", "archive://:3:3//tmp/bar/baz/test.zip"),
			("archive://:2:2//$BAR_BAZ/tmp/test.zip", "archive://:2:2//bar/baz/tmp/test.zip"),
			// -1 component
			("archive:////tmp/test.zip/${BAR/BAZ}", "archive:////tmp/test.zip/bar_baz"),
			("archive://:1//tmp/test.zip/${BAR/BAZ}", "archive://:1//tmp/test.zip/${BAR/BAZ}"),
			("archive://:1//tmp/${BAR/BAZ}/test.zip", "archive://:1//tmp/bar_baz/test.zip"),
			("archive://:2//tmp/test.zip/${BAR/BAZ}", "archive://:1//tmp/test.zip/bar_baz"),
			("archive://:2//tmp/${BAR/BAZ}/test.zip", "archive://:2//tmp/${BAR/BAZ}/test.zip"),
			("archive://:2:1//tmp/test.zip/${BAR/BAZ}", "archive://:2:1//tmp/test.zip/${BAR/BAZ}"),
			("archive://:2:1//tmp/${BAR/BAZ}/test.zip", "archive://:2:1//tmp/${BAR/BAZ}/test.zip"),
			("archive://:2:1//${BAR/BAZ}/tmp/test.zip", "archive://:2:1//bar_baz/tmp/test.zip"),
			("archive://:3:2//tmp/test.zip/${BAR/BAZ}", "archive://:2:1//tmp/test.zip/bar_baz"),
			("archive://:3:2//tmp/${BAR/BAZ}/test.zip", "archive://:3:2//tmp/${BAR/BAZ}/test.zip"),
			("archive://:3:3//tmp/test.zip/${BAR/BAZ}", "archive://:2:2//tmp/test.zip/bar_baz"),
			("archive://:3:3//tmp/${BAR/BAZ}/test.zip", "archive://:2:2//tmp/bar_baz/test.zip"),
			// Zeros all components
			("archive:////${EM/PT/Y}", "archive:////"),
			("archive://:1//${EM/PT/Y}", "archive://:1//${EM/PT/Y}"),
			("archive://:2//${EM/PT/Y}", "archive://:2//${EM/PT/Y}"),
			("archive://:3//${EM/PT/Y}", "archive:////"),
			("archive://:4//${EM/PT/Y}", "archive://:1//"),
		];

		for (input, expected) in cases {
			let u: UrlBuf = input.parse()?;
			assert_eq!(format!("{:?}", expand_url(u).as_url()), expected);
		}

		Ok(())
	}
}
