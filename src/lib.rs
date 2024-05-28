use anyhow::Result;
use std::path::{Path, PathBuf};
use sys_mount::{Mount, MountFlags};

// mod config;
// pub use config::*;

#[inline(always)]
pub fn validate_file(path: impl AsRef<Path>, directory: bool, exists: bool) -> Result<PathBuf> {
	let path = path.as_ref();
	if path.try_exists()? != exists {
		if exists {
			anyhow::bail!("File {path:?} does not exist when it should.")
		} else {
			anyhow::bail!("File {path:?} already exists.")
		};
	}
	if path.is_dir() != directory {
		if directory {
			anyhow::bail!("File {path:?} should not be a directory.")
		} else {
			anyhow::bail!("File {path:?} should be a directory.")
		};
	}
	Ok(path.canonicalize()?)
}
#[inline(always)]
pub fn mount_bind(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<Mount> {
	let from = validate_file(from, false, true)?;
	let to = validate_file(to, true, true)?;
	Ok(Mount::builder().flags(MountFlags::BIND).mount(from, to)?)
}
#[inline(always)]
pub fn mount_fs(
	from: impl AsRef<Path>, to: impl AsRef<Path>, fs: impl AsRef<str>,
) -> Result<Mount> {
	let from = validate_file(from, false, true)?;
	let to = validate_file(to, true, true)?;
	Ok(Mount::builder().fstype(fs.as_ref()).mount(from, to)?)
}
