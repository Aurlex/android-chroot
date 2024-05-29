use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use url::Url;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Args {
	#[command(subcommand)]
	pub command: Command,
	#[arg(short, long)]
	pub root_path: Option<PathBuf>,
}

#[derive(Subcommand, Clone)]
pub enum Command {
	#[command(about = "Creates a chroot in the target directory with a specified RootFS tarball")]
	Install {
		#[arg(short, long)]
		size_root: String,
		#[arg(short, long)]
		url_rootfs: Option<Url>,
		#[arg(short, long)]
		path_rootfs: Option<PathBuf>,
	},
	#[command(about = "SHOULD resize the chroot, but I haven't written it yet")]
	Resize {
		#[arg(short, long)]
		new_size: String,
	},
	#[command(about = "Mount the chroot without starting it")]
	Mount,
	#[command(
		about = "Unmount the chroot. ALWAYS do this after you are finished, unless using start"
	)]
	Umount,
	#[command(
		about = "Mounts, starts, and unmounts the chroot in sucession with the shell of your choosing"
	)]
	Start { shell: Option<PathBuf> },
	#[command(about = "Uninstalls the chroot")]
	Remove,
}

impl Args {
	pub fn validate(self) -> Result<Self> {
		self.root_path.as_ref().context("Root path not set. Set it with -r")?;
		match self.command {
			| Command::Install { ref url_rootfs, ref path_rootfs, .. } => {
				(url_rootfs.is_some() && path_rootfs.is_some())
					.then(|| {})
					.context("One of either url_rootfs or path_rootfs must be set")?;
			},
			| Command::Resize { .. } => {
				bail!("I haven't written the resize operation yet. <3");
			},
			| _ => {},
		}
		Ok(self)
	}
}
