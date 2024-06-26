use anyhow::Result;
use std::path::{Path, PathBuf};
use sys_mount::{Mount, MountFlags};

mod config;
pub use config::*;

#[inline(always)]
pub fn validate_file(
	path: impl AsRef<Path>, directory: Option<bool>, exists: bool,
) -> Result<PathBuf> {
	let path = path.as_ref();
	if path.try_exists()? != exists {
		if exists {
			anyhow::bail!("File {path:?} does not exist when it should.")
		} else {
			anyhow::bail!("File {path:?} already exists.")
		};
	}
	if directory.is_some() && exists && path.is_dir() != directory.unwrap() {
		if directory.unwrap() {
			anyhow::bail!("File {path:?} should be a directory.")
		} else {
			anyhow::bail!("File {path:?} should not be a directory.")
		};
	}
	if exists { Ok(path.canonicalize()?) } else { Ok(path.to_path_buf()) }
}
#[inline(always)]
pub fn mount_loop(
	from: impl AsRef<Path>, to: impl AsRef<Path>, fs: impl AsRef<str>,
) -> Result<Mount> {
	let from = validate_file(from, None, true)?;
	let to = validate_file(to, None, true)?;
	Ok(
		Mount::builder()
			.fstype(fs.as_ref())
			.flags(MountFlags::NOATIME)
			.explicit_loopback()
			.mount(from, to)?,
	)
}
#[inline(always)]
pub fn mount_bind(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<Mount> {
	let from = validate_file(from, None, true)?;
	let to = validate_file(to, None, true)?;
	Ok(Mount::builder().flags(MountFlags::BIND | MountFlags::NOATIME).mount(from, to)?)
}
#[inline(always)]
pub fn mount_fs(
	from: impl AsRef<Path>, to: impl AsRef<Path>, fs: impl AsRef<str>,
) -> Result<Mount> {
	let from = validate_file(from, None, true)?;
	let to = validate_file(to, None, true)?;
	Ok(Mount::builder().fstype(fs.as_ref()).flags(MountFlags::NOATIME).mount(from, to)?)
}
