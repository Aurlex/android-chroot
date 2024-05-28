use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use url::Url;

#[derive(Parser, Serialize, Deserialize)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Arguments {
	#[command(subcommand)]
	command: Option<Subcommands>,
	disk_img_path: Option<PathBuf>,
	root_path: Option<PathBuf>,
	fs_type: Option<String>,
}

#[derive(Subcommand, Clone, Serialize, Deserialize)]
pub enum Subcommands {
	Install {
		url_rootfs: Url,
		path_rootfs: PathBuf,
	},
	Mount,
	Umount,
	Start,
}

impl Arguments {
	pub fn autofill(&mut self, rhs: Self) {
		if self.command.is_none() {
			self.command = rhs.command;
		}
		if self.disk_img_path.is_none() {
			self.disk_img_path = rhs.disk_img_path;
		}
		if self.fs_type.is_none() {
			self.fs_type = rhs.fs_type;
		}
		if self.root_path.is_none() {
			self.root_path = rhs.root_path;
		}
	}
}
