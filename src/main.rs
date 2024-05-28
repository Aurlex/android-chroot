// #[cfg(not(any(target_os = "android", debug_assertions)))]
// compile_error!("Only android is supported");

use android_chroot::{mount_bind, mount_fs, validate_file /* , Arguments */};
use anyhow::Result;
// use clap::Parser;
use flate2::read::GzDecoder;
use loopdev::{LoopControl, LoopDevice};
use std::{
	fs::{create_dir, read_to_string, remove_file, write, File},
	io::copy,
	path::Path,
	process::{Command, Stdio},
	time::Duration,
};
use sys_mount::{unmount, Unmount, UnmountFlags};
use tar::Archive;
use unbytify::unbytify;
use ureq::AgentBuilder;
use url::Url;

fn install(
	root_size: impl AsRef<str>, url_tar_rootfs: Option<Url>, path_tar_rootfs: impl AsRef<Path>,
	root_path: impl AsRef<Path>,
) -> Result<()> {
	let root_path = validate_file(root_path, false, false)?;
	let img_path = validate_file(root_path.parent().unwrap().join("disk.img"), false, false)?;
	let mut path_tar_rootfs = path_tar_rootfs.as_ref().to_path_buf();
	create_dir(&root_path)?;
	if let Some(url_rootfs) = url_tar_rootfs {
		path_tar_rootfs = root_path.parent().unwrap().join("rootfs.tar.gz");
		let mut tar = File::create(&path_tar_rootfs)?;
		copy(
			&mut AgentBuilder::new()
				.timeout_read(Duration::from_secs(5))
				.timeout_write(Duration::from_secs(5))
				.build()
				.get(url_rootfs.as_ref())
				.call()?
				.into_reader(),
			&mut tar,
		)?;
	}
	let tar_gz = File::open(path_tar_rootfs)?;
	let tar = GzDecoder::new(tar_gz);
	let mut archive = Archive::new(tar);

	let size_bytes = unbytify(root_size.as_ref())?;
	let disk_img = File::create(&img_path)?;
	disk_img.set_len(size_bytes)?;
	Command::new("mke2fs")
		.args(["-t", "ext4", img_path.to_str().unwrap()])
		.stderr(Stdio::null())
		.stdout(Stdio::null())
		.spawn()?
		.wait()?;
	let loop_device = LoopControl::open()?.next_free()?;
	loop_device.attach_file(&img_path)?;
	let _mount = mount_fs(&img_path, &root_path, "ext4")?.into_unmount_drop(UnmountFlags::EXPIRE);
	archive.unpack(&root_path)?;
	create_dir(root_path.join("sdcard"))?;
	Ok(())
}

// TODO: this
// fn resize(new_size: &String, root_path: impl AsRef<Path>) -> Result<()> {
// 	let root_path = validate_file(root_path, true, true)?;
// 	let img_path = validate_file(root_path.parent().unwrap().join("disk.img"),
// false, true)?;

// 	validate_file(root_path.parent().unwrap().join("loopdevice.lock"), false,
// false)?;

// 	Ok(())
// }

fn mount(root_path: impl AsRef<Path>) -> Result<()> {
	let root_path = validate_file(root_path, true, true)?;
	let img_path = validate_file(root_path.parent().unwrap().join("disk.img"), false, true)?;
	let loop_device = LoopControl::open()?.next_free()?;
	loop_device.attach_file(&img_path)?;
	mount_fs(&img_path, &root_path, "ext4")?;
	mount_bind("/dev", &root_path.join("dev"))?;
	mount_fs("proc", &root_path.join("proc"), "proc")?;
	mount_fs("sysfs", &root_path.join("sys"), "sysfs")?;
	mount_fs("tmpfs", &root_path.join("tmp"), "tmpfs")?;
	mount_fs("devpts", &root_path.join("dev/pts"), "devpts")?;
	mount_bind("/sdcard", &root_path.join("sdcard"))?;
	write(
		root_path.parent().unwrap().join("loopdevice.lock"),
		loop_device.path().unwrap().to_str().unwrap(),
	)?;
	Ok(())
}

fn umount(root_path: impl AsRef<Path>) -> Result<()> {
	let root_path = validate_file(root_path, true, true)?;
	unmount(&root_path.join("sdcard"), UnmountFlags::EXPIRE)?;
	unmount(&root_path.join("dev/pts"), UnmountFlags::EXPIRE)?;
	unmount(&root_path.join("tmp"), UnmountFlags::EXPIRE)?;
	unmount(&root_path.join("sys"), UnmountFlags::EXPIRE)?;
	unmount(&root_path.join("proc"), UnmountFlags::EXPIRE)?;
	unmount(&root_path.join("dev"), UnmountFlags::EXPIRE)?;
	unmount(&root_path.join("root"), UnmountFlags::EXPIRE)?;
	let lock_path = root_path.parent().unwrap().join("loopdevice.lock");
	let loop_device = LoopDevice::open(read_to_string(&lock_path)?)?;
	loop_device.detach()?;
	remove_file(lock_path)?;
	Ok(())
}

fn start(
	root_path: impl AsRef<Path>, user: impl AsRef<str>, shell: impl AsRef<Path>,
) -> Result<()> {
	let root_path = validate_file(root_path, true, true)?;
	validate_file(root_path.join(shell.as_ref().strip_prefix("/")?), false, true)?;
	mount(&root_path)?;
	Command::new("unshare")
		.env_clear()
		.env("TERM", "xterm-256color")
		.args([
			"--uts",
			"env",
			"-i",
			"chroot",
			"-u",
			user.as_ref(),
			root_path.to_str().unwrap(),
			shell.as_ref().to_str().unwrap(),
		])
		.stderr(Stdio::inherit())
		.stdin(Stdio::inherit())
		.stdout(Stdio::inherit())
		.spawn()?
		.wait()?;
	println!("done");
	umount(root_path)?;
	Ok(())
}

fn main() -> Result<()> {
	install(
		"10G",
		Some("https://fl.us.mirror.archlinuxarm.org/os/ArchLinuxARM-aarch64-latest.tar.gz".try_into()?),
		"",
		"./root",
	)
}
