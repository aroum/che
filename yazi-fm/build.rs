use std::{env, error::Error};

fn main() -> Result<(), Box<dyn Error>> {
	let dir = env::var("OUT_DIR").unwrap();

	// cargo build
	//   C:\Users\Ika\Desktop\yazi\target\release\build\yazi-fm-cfc94820f71daa30\out
	// cargo install
	//   C:\Users\Ika\AppData\Local\Temp\cargo-installTFU8cj\release\build\
	// yazi-fm-45dffef2500eecd0\out

	if dir.contains(r"\release\build\gezi-fm-") {
		panic!(
			"Unwinding must be enabled for Windows. Please use `cargo build --profile release-windows --locked` instead to build Gezi."
		);
	}

	let manifest = env::var_os("CARGO_MANIFEST_DIR").unwrap().to_string_lossy().replace(r"\", "/");
	if env::var_os("GEZI_CRATE_BUILD").is_none()
		&& (manifest.contains("/git/checkouts/gezi-")
			|| manifest.contains("/registry/src/index.crates.io-"))
	{
		panic!(
			"Due to Cargo's limitations, the `gezi-fm` and `gezi-cli` crates on crates.io must be built with `cargo install --force gezi-build`"
		);
	}

	Ok(())
}
