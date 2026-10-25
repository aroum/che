#![allow(clippy::all, unused)]

yazi_macro::mod_pub!(dds package shared);

yazi_macro::mod_flat!(args);
mod archive;
mod input;
mod multirename;

use std::process::ExitCode;

use clap::Parser;
use yazi_macro::{errln, outln};
use yazi_shared::LOCAL_SET;

#[tokio::main]
async fn main() -> ExitCode {
	yazi_shared::init();
	yazi_fs::init();

	match LOCAL_SET.run_until(run()).await {
		Ok(()) => ExitCode::SUCCESS,
		Err(e) => {
			for cause in e.chain() {
				if let Some(ioerr) = cause.downcast_ref::<std::io::Error>()
					&& ioerr.kind() == std::io::ErrorKind::BrokenPipe
				{
					return ExitCode::from(0);
				}
			}
			errln!("{e:#}").ok();
			ExitCode::FAILURE
		}
	}
}

async fn run() -> anyhow::Result<()> {
	yazi_version::setup!();

	match Args::parse().command {
		Command::Emit(cmd) => {
			yazi_boot::init_default();
			yazi_dds::init();
			if let Err(e) = dds::Dds::shot("dds-emit", CommandPub::receiver()?, &cmd.body()?).await {
				errln!("Cannot emit command: {e}")?;
				std::process::exit(1);
			}
		}

		Command::EmitTo(cmd) => {
			yazi_boot::init_default();
			yazi_dds::init();
			if let Err(e) = dds::Dds::shot("dds-emit", cmd.receiver, &cmd.body()?).await {
				errln!("Cannot emit command: {e}")?;
				std::process::exit(1);
			}
		}

		Command::Exec(cmd) => {
			yazi_boot::init_default();
			yazi_dds::init();

			match dds::Dds::exec(cmd).await {
				Ok(data) => outln!("{}", serde_json::to_string(&data)?)?,
				Err(e) => {
					errln!("Cannot execute command: {e}")?;
					std::process::exit(1);
				}
			}
		}

		Command::Pkg(cmd) => {
			package::init()?;

			let mut pkg = package::Package::load().await?;
			match cmd {
				CommandPkg::Add { ids } => pkg.add_many(&ids).await?,
				CommandPkg::Delete { ids } => pkg.delete_many(&ids).await?,
				CommandPkg::Install => pkg.install().await?,
				CommandPkg::List => pkg.print()?,
				CommandPkg::Upgrade { ids } => pkg.upgrade_many(&ids).await?,
			}
		}

		Command::Pub(cmd) => {
			yazi_boot::init_default();
			yazi_dds::init();
			if let Err(e) = dds::Dds::shot(&cmd.kind, CommandPub::receiver()?, &cmd.body()?).await {
				errln!("Cannot send message: {e}")?;
				std::process::exit(1);
			}
		}

		Command::PubTo(cmd) => {
			yazi_boot::init_default();
			yazi_dds::init();
			if let Err(e) = dds::Dds::shot(&cmd.kind, cmd.receiver, &cmd.body()?).await {
				errln!("Cannot send message: {e}")?;
				std::process::exit(1);
			}
		}

		Command::Sub(cmd) => {
			yazi_boot::init_default();
			yazi_dds::init();
			dds::Dds::draw(cmd.kinds.split(',').collect()).await?;

			tokio::signal::ctrl_c().await?;
		}

		Command::Multirename(cmd) => {
			multirename::run(cmd.files)?;
		}

		Command::Archive(cmd) => {
			archive::run(cmd.files, cmd.mode, cmd.output_dir, cmd.opposite_dir)?;
		}

		Command::Env => {
			let che_bin = std::env::current_exe()
				.ok()
				.and_then(|p| {
					let sibling = p.with_file_name(if cfg!(windows) { "che.exe" } else { "che" });
					if sibling.is_file() { Some(sibling) } else { None }
				})
				.unwrap_or_else(|| std::path::PathBuf::from(if cfg!(windows) { "che.exe" } else { "che" }));

			let status = std::process::Command::new(che_bin).arg("--debug").status();

			match status {
				Ok(s) => {
					if !s.success() {
						std::process::exit(s.code().unwrap_or(1));
					}
				}
				Err(e) => {
					errln!("Cannot run 'che --debug': {e}")?;
					std::process::exit(1);
				}
			}
		}
	}

	Ok(())
}
