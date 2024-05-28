use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::{
	fs::read_to_string,
	io::{Error, ErrorKind, Result},
	path::{Path, PathBuf},
};
use sys_mount::SupportedFilesystems;
use url::Url;

use crate::validate_file;

#[derive(Parser, Serialize, Deserialize, Debug, Clone)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Arguments {
	#[command(subcommand)]
	pub command: Option<Subcommands>,
	#[arg(short, long)]
	pub disk_img_path: Option<PathBuf>,
	#[arg(short, long)]
	pub root_path: Option<PathBuf>,
	#[arg(short, long)]
	pub fs_type: Option<String>,
}

#[derive(Subcommand, Clone, Serialize, Deserialize, Debug)]
pub enum Subcommands {
	Resize {
		#[arg(short, long)]
		new_size: String,
	},
	Install {
		#[arg(short, long)]
		size_root: String,
		#[arg(short, long)]
		url_rootfs: Option<Url>,
		#[arg(short, long, default_value = "./rootfs")]
		path_rootfs: PathBuf,
	},
	Mount,
	Umount,
	Start,
}

impl Arguments {
	pub const fn default() -> Self {
		Self {
			command: None,
			disk_img_path: None,
			root_path: None,
			fs_type: None,
		}
	}
	pub fn autofill(self, rhs: Self) -> Self {
		Self {
			command: self.command.or(rhs.command),
			disk_img_path: self.disk_img_path.or(rhs.disk_img_path),
			fs_type: self.fs_type.or(rhs.fs_type),
			root_path: self.root_path.or(rhs.root_path),
		}
	}
	pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self> {
		Ok(toml::from_str(
			&read_to_string(path.as_ref())
				.unwrap_or_else(|_| toml::to_string(&Self::default()).unwrap()),
		)
		.unwrap())
	}
	pub fn validate(self) -> Result<Self> {
		(self.command.is_some()
			|| (self.disk_img_path.is_some()
				&& validate_file(self.disk_img_path.as_ref().unwrap(), false, true).is_ok())
			|| (self.fs_type.is_some()
				&& SupportedFilesystems::new()?.is_supported(self.fs_type.as_ref().unwrap()))
			|| (self.root_path.is_some()
				&& validate_file(self.root_path.as_ref().unwrap(), true, true).is_ok()))
		.then(|| self.clone())
		.ok_or(Error::new(
			ErrorKind::InvalidInput,
			format!("Arguments are incomplete: {self:?}"),
		))
	}
}
