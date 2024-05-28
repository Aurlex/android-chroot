// #[cfg(not(any(target_os = "android", debug_assertions)))]
// compile_error!("Only android is supported");

use android_chroot::{mount_bind, mount_fs, mount_loop, validate_file /* , Arguments */};
use anyhow::Result;
// use clap::Parser;
use flate2::bufread::GzDecoder;
use loopdev::{LoopControl, LoopDevice};
use std::{
	fs::{create_dir, read_to_string, remove_file, write, File},
	io::{copy, BufReader, BufWriter},
	path::Path,
	process::{Command, Stdio},
	thread::sleep,
	time::Duration,
};
use sys_mount::{unmount, Unmount, UnmountFlags};
use tar::Archive;
use unbytify::{bytify, unbytify};
use ureq::get;
use url::Url;

fn install(
	root_size: impl AsRef<str>, url_tar_rootfs: Option<Url>, path_tar_rootfs: impl AsRef<Path>,
	root_path: impl AsRef<Path>,
) -> Result<()> {
	let root_path = validate_file(root_path, Some(false), false)?;
	let img_path = validate_file(root_path.parent().unwrap().join("disk.img"), Some(false), false)?;
	let mut path_tar_rootfs = path_tar_rootfs.as_ref().to_path_buf();
	if let Some(url_rootfs) = url_tar_rootfs {
		let request = get(url_rootfs.as_ref())
			.set(
				"User-Agent",
				"Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:95.0) Gecko/20100101 Firefox/95.0",
			)
			.call()?;
		let url_path = "filename=".to_owned() + url_rootfs.path_segments().unwrap().last().unwrap();
		let file_name =
			request.header("Content-Disposition").unwrap_or(&url_path).split("filename=").last().unwrap();
		let (file_size, ext) = bytify(request.header("Content-Length").unwrap().parse()?);
		path_tar_rootfs = root_path.parent().unwrap().join(file_name);
		let path = path_tar_rootfs.clone();
		let mut tar = File::create(&path)?;
		println!("Downloading: {file_name}, {file_size}{ext}.");
		copy(&mut request.into_reader(), &mut BufWriter::new(&mut tar))?;
	}
	let tar_gz = BufReader::new(File::open(path_tar_rootfs)?);
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
	create_dir(&root_path)?;
	// loop_device.attach_file(&img_path)?;
	let mount = mount_loop(&img_path, &root_path, "ext4")?;
	// let loop_device = mount.backing_loop_device().unwrap();
	let (loop_device, automounted) = if mount.backing_loop_device().is_none() {
		let loop_device = LoopControl::open()?.next_free()?;
		loop_device.attach_file(&img_path)?;
		(loop_device, false)
	} else {
		(LoopDevice::open(mount.backing_loop_device().unwrap())?, true)
	};
	archive.unpack(&root_path)?;
	create_dir(root_path.join("sdcard"))?;
	mount.unmount(UnmountFlags::DETACH)?;
	// Not sure what to do about the spin down time.
	if !automounted {
		sleep(Duration::from_millis(400));
		loop_device.detach()?;
	}
	Ok(())
}

// TODO: this
fn resize(new_size: impl AsRef<str>, root_path: impl AsRef<Path>) -> Result<()> {
	let root_path = validate_file(root_path, Some(true), true)?;
	let img_path = validate_file(root_path.parent().unwrap().join("disk.img"), Some(false), true)?;
	let new_size = unbytify(new_size.as_ref())?;
	validate_file(root_path.parent().unwrap().join("loopdevice.lock"), Some(false), false)?;

	Ok(())
}

fn mount(root_path: impl AsRef<Path>) -> Result<bool> {
	let root_path = validate_file(root_path, Some(true), true)?;
	let img_path = validate_file(root_path.parent().unwrap().join("disk.img"), Some(false), true)?;
	let mount = mount_loop(&img_path, &root_path, "ext4")?;
	let (loop_device, automounted) = if mount.backing_loop_device().is_none() {
		let loop_device = LoopControl::open()?.next_free()?;
		loop_device.attach_file(&img_path)?;
		(loop_device, false)
	} else {
		(LoopDevice::open(mount.backing_loop_device().unwrap())?, true)
	};
	mount_bind("/dev", &root_path.join("dev"))?;
	mount_fs("/proc", &root_path.join("proc"), "proc")?;
	mount_fs("/sys", &root_path.join("sys"), "sysfs")?;
	mount_fs("/data/local/tmp", &root_path.join("tmp"), "tmpfs")?;
	mount_fs("/dev/pts", &root_path.join("dev/pts"), "devpts")?;
	mount_bind("/sdcard", &root_path.join("sdcard"))?;
	write(
		root_path.parent().unwrap().join("loopdevice.lock"),
		loop_device.path().unwrap().to_str().unwrap(),
	)?;
	Ok(automounted)
}

fn umount(root_path: impl AsRef<Path>, automounted: bool) -> Result<()> {
	let root_path = validate_file(root_path, Some(true), true)?;
	unmount(&root_path.join("sdcard"), UnmountFlags::DETACH)?;
	unmount(&root_path.join("dev/pts"), UnmountFlags::DETACH)?;
	unmount(&root_path.join("tmp"), UnmountFlags::DETACH)?;
	unmount(&root_path.join("sys"), UnmountFlags::DETACH)?;
	unmount(&root_path.join("proc"), UnmountFlags::DETACH)?;
	unmount(&root_path.join("dev"), UnmountFlags::DETACH)?;
	unmount(&root_path, UnmountFlags::DETACH)?;
	let lock_path = root_path.parent().unwrap().join("loopdevice.lock");
	let loop_device = LoopDevice::open(read_to_string(&lock_path)?)?;
	sleep(Duration::from_millis(400));
	loop_device.detach()?;
	remove_file(lock_path)?;
	Ok(())
}

fn start(
	root_path: impl AsRef<Path>, user: impl AsRef<str>, shell: impl AsRef<Path>,
) -> Result<()> {
	let root_path = validate_file(root_path, Some(true), true)?;
	let automounted = mount(&root_path)?;
	validate_file(root_path.join(shell.as_ref().strip_prefix("/")?), Some(false), true)?;
	Command::new("chroot")
		.env_clear()
		.env("TERM", "xterm-256color")
		.args([root_path.as_ref(), shell.as_ref()])
		.stderr(Stdio::inherit())
		.stdin(Stdio::inherit())
		.stdout(Stdio::inherit())
		.spawn()?
		.wait()?;
	println!("done");
	umount(root_path, automounted)?;
	Ok(())
}

fn main() -> Result<()> {
	install(
		"10G",
		// Some("https://fl.us.mirror.archlinuxarm.org/os/ArchLinuxARM-aarch64-latest.tar.gz".try_into()?),
		None,
		"./ArchLinuxARM-aarch64-latest.tar.gz",
		"./root",
	)?;
	start("./root", "root", "/bin/sh")
}
