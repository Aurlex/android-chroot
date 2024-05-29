#[cfg(not(any(target_os = "android", debug_assertions)))]
compile_error!("Only android is supported");

use android_chroot::{mount_bind, mount_fs, mount_loop, validate_file, Args};
use anyhow::{bail, Result};
use clap::Parser;
use flate2::bufread::GzDecoder;
use loopdev::{LoopControl, LoopDevice};
use std::{
	fs::{create_dir, read_to_string, remove_dir, remove_file, write, File},
	io::{copy, BufReader, BufWriter, Read},
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
	root_size: impl AsRef<str>, url_tar_rootfs: Option<Url>,
	path_tar_rootfs: Option<impl AsRef<Path>>, root_path: impl AsRef<Path>,
) -> Result<()> {
	let root_path = validate_file(root_path, Some(false), false)?;
	let img_path = validate_file(root_path.parent().unwrap().join("disk.img"), Some(false), false)?;
	if path_tar_rootfs.is_none() && url_tar_rootfs.is_none() {
		bail!("Either path_rootfs or url_rootfs must be set.")
	}
	let path;
	if let Some(url_rootfs) = url_tar_rootfs {
		let request = get(url_rootfs.as_ref()).call()?;
		let url_path = "filename=".to_owned() + url_rootfs.path_segments().unwrap().last().unwrap();
		let file_name =
			request.header("Content-Disposition").unwrap_or(&url_path).split("filename=").last().unwrap();
		let (file_size, ext) = bytify(request.header("Content-Length").unwrap().parse()?);
		path = root_path.parent().unwrap().join(file_name);
		// let path = path_tar_rootfs.clone();
		let mut tar = File::create(&path)?;
		println!("Downloading: {file_name}, {file_size}{ext}.");
		copy(&mut request.into_reader(), &mut BufWriter::new(&mut tar))?;
		println!("Finished Download.");
	} else {
		path = path_tar_rootfs.unwrap().as_ref().to_path_buf();
	}
	let tar_gz = BufReader::new(File::open(path)?);
	let tar = GzDecoder::new(tar_gz);
	let mut archive = Archive::new(tar);

	let size_bytes = unbytify(root_size.as_ref())?;
	let disk_img = File::create(&img_path)?;
	disk_img.set_len(size_bytes)?;
	println!("Formatting disk.img");
	Command::new("mke2fs")
		.args(["-t", "ext4", img_path.to_str().unwrap()])
		.stderr(Stdio::null())
		.stdout(Stdio::null())
		.spawn()?
		.wait()?;
	Command::new("tune2fs")
		.args(["-o", "journal_data_writeback", img_path.to_str().unwrap()])
		.stderr(Stdio::null())
		.stdout(Stdio::null())
		.spawn()?
		.wait()?;
	Command::new("tune2fs")
		.args(["-O", "^has_journal", img_path.to_str().unwrap()])
		.stderr(Stdio::null())
		.stdout(Stdio::null())
		.spawn()?
		.wait()?;
	Command::new("e2fsck")
		.args(["-f", img_path.to_str().unwrap()])
		.stderr(Stdio::null())
		.stdout(Stdio::null())
		.spawn()?
		.wait()?;
	create_dir(&root_path)?;
	println!("Mounting disk.img");
	let mount = mount_loop(&img_path, &root_path, "ext4")?;
	let (loop_device, automounted) = if mount.backing_loop_device().is_none() {
		let loop_device = LoopControl::open()?.next_free()?;
		loop_device.attach_file(&img_path)?;
		(loop_device, false)
	} else {
		(LoopDevice::open(mount.backing_loop_device().unwrap())?, true)
	};
	println!("Unpacking RootFS");
	archive.unpack(&root_path)?;
	create_dir(root_path.join("sdcard"))?;
	remove_file(root_path.join("etc/resolv.conf"))?;
	write("etc/hosts.conf", read_to_string("/etc/hosts.conf")?)?;
	write("etc/hostname.conf", read_to_string("/etc/hostname.conf")?)?;
	println!("Unomunting disk.img");
	mount.unmount(UnmountFlags::DETACH)?;
	// Not sure what to do about the spin down time.
	if !automounted {
		sleep(Duration::from_millis(400));
		loop_device.detach()?;
	}
	println!("Done");
	Ok(())
}

// TODO: this
fn _resize(_new_size: impl AsRef<str>, _root_path: impl AsRef<Path>) -> Result<()> { todo!() }

fn mount(root_path: impl AsRef<Path>) -> Result<()> {
	let root_path = validate_file(root_path, Some(true), true)?;
	let img_path = validate_file(root_path.parent().unwrap().join("disk.img"), Some(false), true)?;
	println!("Mounting disk.img");
	let mount = mount_loop(&img_path, &root_path, "ext4")?;
	let loop_device = if mount.backing_loop_device().is_none() {
		let loop_device = LoopControl::open()?.next_free()?;
		loop_device.attach_file(&img_path)?;
		loop_device
	} else {
		LoopDevice::open(mount.backing_loop_device().unwrap())?
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
	Command::new("fstrim")
		.args(["-av", img_path.to_str().unwrap()])
		.stderr(Stdio::null())
		.stdout(Stdio::null())
		.spawn()?
		.wait()?;
	println!("Done");
	Ok(())
}

fn umount(root_path: impl AsRef<Path>) -> Result<()> {
	let root_path = validate_file(root_path, Some(true), true)?;
	let lock_path =
		validate_file(root_path.parent().unwrap().join("loopdevice.lock"), Some(false), true)?;
	let loop_device = LoopDevice::open(read_to_string(&lock_path)?)?;
	println!("Unmounting: sdcard");
	unmount(&root_path.join("sdcard"), UnmountFlags::DETACH)?;
	println!("Unmounting: dev/pts");
	unmount(&root_path.join("dev/pts"), UnmountFlags::DETACH)?;
	println!("Unmounting: tmp");
	unmount(&root_path.join("tmp"), UnmountFlags::DETACH)?;
	println!("Unmounting: sys");
	unmount(&root_path.join("sys"), UnmountFlags::DETACH)?;
	println!("Unmounting: proc");
	unmount(&root_path.join("proc"), UnmountFlags::DETACH)?;
	println!("Unmounting: dev");
	unmount(&root_path.join("dev"), UnmountFlags::DETACH)?;
	println!("Unmounting: root");
	unmount(&root_path, UnmountFlags::DETACH)?;
	sleep(Duration::from_millis(400));
	loop_device.detach()?;
	remove_file(lock_path)?;
	println!("Done");
	Ok(())
}

fn start(
	root_path: impl AsRef<Path>, _user: impl AsRef<str>, shell: impl AsRef<Path>,
) -> Result<()> {
	let root_path = validate_file(root_path, Some(true), true)?;
	mount(&root_path)?;
	validate_file(root_path.join(shell.as_ref().strip_prefix("/")?), Some(false), true)?;
	println!("Starting chroot");
	Command::new("chroot")
		.env_clear()
		.env("TERM", "xterm-256color")
		.args([root_path.as_ref(), shell.as_ref()])
		.stderr(Stdio::inherit())
		.stdin(Stdio::inherit())
		.stdout(Stdio::inherit())
		.spawn()?
		.wait()?;
	println!("Done");
	umount(root_path)?;
	Ok(())
}

fn remove(root_path: impl AsRef<Path>) -> Result<()> {
	let root_path = validate_file(root_path, Some(true), true)?;
	let img_path = validate_file(root_path.parent().unwrap().join("disk.img"), Some(false), true)?;
	validate_file(root_path.parent().unwrap().join("loopdevice.lock"), Some(false), false)?;
	let mut buf = [0u8];
	println!("Are you sure you want to delete {}? [y/N]", root_path.to_str().unwrap());
	std::io::stdin().read_exact(&mut buf)?;
	match buf[0] as char {
		| 'y' | 'Y' => {
			println!("Deleting");
			remove_dir(root_path)?;
			remove_file(img_path)?;
		},
		| _ => println!("Not deleting"),
	}
	println!("Done");
	Ok(())
}

fn main() -> Result<()> {
	let args = Args::parse().validate()?;
	use android_chroot::Command::*;
	match args.command {
		| Install { ref size_root, ref url_rootfs, ref path_rootfs } => {
			install(size_root, url_rootfs.clone(), path_rootfs.clone(), &args.root_path.unwrap())
		},
		| Mount => mount(&args.root_path.unwrap()),
		| Umount => umount(&args.root_path.unwrap()),
		| Start { ref shell } => {
			start(&args.root_path.unwrap(), "", shell.as_ref().unwrap_or(&"/bin/bash".try_into()?))
		},
		| Remove => remove(&args.root_path.unwrap()),
		| _ => bail!("How did you get here?"),
	}
}
