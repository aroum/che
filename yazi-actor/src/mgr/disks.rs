use anyhow::Result;
use yazi_config::{YAZI, popup::{Offset, PickCfg, Position}};
use yazi_fs::path::expand_url;
use yazi_macro::succ;
use yazi_parser::VoidOpt;
use yazi_proxy::{MgrProxy, PickProxy};
use yazi_shared::{data::Data, url::{UrlBuf, UrlLike}};

use crate::{Actor, Ctx};

pub struct Disks;

impl Actor for Disks {
	type Options = VoidOpt;

	const NAME: &str = "disks";

	fn act(_: &mut Ctx, _: Self::Options) -> Result<Data> {
		let disks = Self::get_disks();
		if disks.is_empty() {
			succ!();
		}

		let items: Vec<String> = disks.iter().map(|d| d.display.clone()).collect();
		let targets: Vec<UrlBuf> = disks.into_iter().map(|d| d.path).collect();

		tokio::spawn(async move {
			let max_height = YAZI.pick.open_offset.height.min(YAZI.pick.border().saturating_add(items.len() as u16));
			let cfg = PickCfg {
				title: "Select Disk / Volume".to_string(),
				items,
				position: Position::new(YAZI.pick.open_origin, Offset {
					height: max_height,
					..YAZI.pick.open_offset
				}),
			};
			if let Some(idx) = PickProxy::show(cfg).await {
				if let Some(target) = targets.get(idx) {
					MgrProxy::cd(target);
				}
			}
		});

		succ!();
	}
}

struct DiskInfo {
	display: String,
	path:    UrlBuf,
}

impl Disks {
	fn get_disks() -> Vec<DiskInfo> {
		let mut disks = Vec::new();

		// Home directory
		let home_url: UrlBuf = expand_url(std::path::Path::new("~")).into();
		disks.push(DiskInfo {
			display: format!("~ (Home: {})", home_url.display()),
			path: home_url,
		});

		// Root directory
		#[cfg(unix)]
		{
			disks.push(DiskInfo {
				display: "/ (Root)".to_string(),
				path: UrlBuf::from(std::path::Path::new("/")),
			});
		}

		// Windows drives
		#[cfg(windows)]
		{
			for letter in 'A'..='Z' {
				let path_str = format!("{}:\\", letter);
				let path = std::path::Path::new(&path_str);
				if path.exists() {
					disks.push(DiskInfo {
						display: format!("Drive {}:", letter),
						path: UrlBuf::from(path),
					});
				}
			}
		}

		// macOS Volumes
		#[cfg(target_os = "macos")]
		{
			if let Ok(entries) = std::fs::read_dir("/Volumes") {
				for entry in entries.flatten() {
					let p = entry.path();
					if p.is_dir() {
						let name = p.file_name().unwrap_or_default().to_string_lossy().into_owned();
						disks.push(DiskInfo {
							display: format!("Volume: {}", name),
							path: UrlBuf::from(p.as_path()),
						});
					}
				}
			}
		}

		// Linux mounts
		#[cfg(target_os = "linux")]
		{
			let mut candidate_bases = vec![
				std::path::PathBuf::from("/media"),
				std::path::PathBuf::from("/mnt"),
			];
			if let Ok(user) = std::env::var("USER") {
				candidate_bases.push(std::path::PathBuf::from(format!("/media/{}", user)));
				candidate_bases.push(std::path::PathBuf::from(format!("/run/media/{}", user)));
			}

			let mut seen = std::collections::HashSet::new();
			for base in candidate_bases {
				if let Ok(entries) = std::fs::read_dir(&base) {
					for entry in entries.flatten() {
						let p = entry.path();
						if p.is_dir() && seen.insert(p.clone()) {
							let name = p.file_name().unwrap_or_default().to_string_lossy().into_owned();
							disks.push(DiskInfo {
								display: format!("Mount: {} ({})", name, base.display()),
								path: UrlBuf::from(p.as_path()),
							});
						}
					}
				}
			}
		}

		disks
	}
}
