use android_chroot::Arguments;
use clap::Parser;
use std::fs::{write, File};
use std::io::Result;
use std::path::PathBuf;
use unbytify::unbytify;
use url::Url;

#[cfg(not(any(target_os = "android", debug_assertions)))]
compile_error!("Only android is supported");

fn install(
	root_size: &String,
	url_rootfs: &Option<Url>,
	path_rootfs: &PathBuf,
	disk_img_path: PathBuf,
	fs_type: String,
	root_path: PathBuf,
) -> Result<()> {
	let size_bytes = unbytify(&root_size).expect(&format!("Could not parse: \"{}\"", root_size));
	println!("{size_bytes} bytes");
	let file = File::create(disk_img_path)?;
	file.set_len(size_bytes)?;
	Ok(())
}

fn resize(
	new_size: &String,
	disk_img_path: PathBuf,
	fs_type: String,
	root_path: PathBuf,
) -> Result<()> {
	todo!()
}

fn mount(disk_img_path: PathBuf, fs_type: String, root_path: PathBuf) -> Result<()> {
	todo!()
}

fn umount(disk_img_path: PathBuf, fs_type: String, root_path: PathBuf) -> Result<()> {
	todo!()
}

fn start(disk_img_path: PathBuf, fs_type: String, root_path: PathBuf) -> Result<()> {
	todo!()
}

fn main() -> Result<()> {
	let args = Arguments::parse()
		.autofill(
			Arguments::load_from_file("./android-chroot.toml")?.autofill(
				Arguments::load_from_file("$HOME/.config/android-chroot.toml")?,
			),
		)
		.validate()?;
	let mut args2 = args.clone();
	args2.command = None;
	match args.command.as_ref().unwrap() {
		android_chroot::Subcommands::Install {
			size_root,
			url_rootfs,
			path_rootfs,
		} => {
			write("./android-chroot.toml", toml::to_string(&args2).unwrap())?;
			install(
				size_root,
				url_rootfs,
				path_rootfs,
				args.disk_img_path.unwrap(),
				args.fs_type.unwrap(),
				args.root_path.unwrap(),
			)
		}
		android_chroot::Subcommands::Resize { new_size } => resize(
			new_size,
			args.disk_img_path.unwrap(),
			args.fs_type.unwrap(),
			args.root_path.unwrap(),
		),
		android_chroot::Subcommands::Mount => mount(
			args.disk_img_path.unwrap(),
			args.fs_type.unwrap(),
			args.root_path.unwrap(),
		),
		android_chroot::Subcommands::Umount => umount(
			args.disk_img_path.unwrap(),
			args.fs_type.unwrap(),
			args.root_path.unwrap(),
		),
		android_chroot::Subcommands::Start => start(
			args.disk_img_path.unwrap(),
			args.fs_type.unwrap(),
			args.root_path.unwrap(),
		),
	}
}
