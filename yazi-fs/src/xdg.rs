use std::{env, path::PathBuf, sync::OnceLock};

pub struct Xdg;

impl Xdg {
	pub fn config_dir() -> PathBuf {
		if let Some(p) = env::var_os("CHE_CONFIG_HOME").or_else(|| env::var_os("GEZI_CONFIG_HOME")).map(PathBuf::from)
			&& p.is_absolute()
		{
			return p;
		}

		#[cfg(windows)]
		{
			dirs::config_dir()
				.map(|p| p.join("che").join("config"))
				.expect("Failed to get config directory")
		}
		#[cfg(unix)]
		{
			env::var_os("XDG_CONFIG_HOME")
				.map(PathBuf::from)
				.filter(|p| p.is_absolute())
				.or_else(|| dirs::home_dir().map(|h| h.join(".config")))
				.map(|p| p.join("che"))
				.expect("Failed to get config directory")
		}
	}

	pub fn state_dir() -> PathBuf {
		#[cfg(windows)]
		{
			dirs::data_dir().map(|p| p.join("che").join("state")).expect("Failed to get state directory")
		}
		#[cfg(unix)]
		{
			env::var_os("XDG_STATE_HOME")
				.map(PathBuf::from)
				.filter(|p| p.is_absolute())
				.or_else(|| dirs::home_dir().map(|h| h.join(".local/state")))
				.map(|p| p.join("che"))
				.expect("Failed to get state directory")
		}
	}

	pub fn cache_dir() -> &'static PathBuf {
		static CACHE: OnceLock<PathBuf> = OnceLock::new();

		CACHE.get_or_init(|| {
			let mut p = env::temp_dir();
			assert!(p.is_absolute(), "Temp dir is not absolute");

			#[cfg(unix)]
			{
				use uzers::Users;
				p.push(format!("che-{}", yazi_shared::USERS_CACHE.get_current_uid()))
			}
			#[cfg(not(unix))]
			p.push("che");

			p
		})
	}
}

