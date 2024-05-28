use std::fs::{read_to_string, ReadDir};
use std::io::{Error, ErrorKind, Result};
use std::path::{Path, PathBuf};
use sys_mount::{Mount, SupportedFilesystems};

mod config;

#[cfg(test)]
mod tests;

#[inline(always)]
pub fn validate_file(path: impl AsRef<Path>, directory: bool) -> Result<PathBuf> {
	let path = path.as_ref().canonicalize()?;
	if !path.try_exists()? {
		return Err(Error::new(
			ErrorKind::NotFound,
			format!("File \"{path:?}\" not found."),
		));
	}
	if path.is_dir() != directory {
		return Err(Error::new(
			ErrorKind::InvalidInput,
			format!(
				"File {path:?} is {}a directory.",
				if directory { "not " } else { "" }
			),
		));
	}
	Ok(path)
}
#[inline(always)]
pub fn ls(path: impl AsRef<Path>) -> Result<ReadDir> {
	validate_file(path, true)?.read_dir()
}
#[inline(always)]
pub fn cat(path: impl AsRef<Path>) -> Result<String> {
	read_to_string(validate_file(path, false)?)
}
#[inline(always)]
pub fn mount_loopback(
	from: impl AsRef<Path>,
	to: impl AsRef<Path>,
	fstype: impl AsRef<str>,
) -> Result<Mount> {
	let from = validate_file(from, false)?;
	let to = validate_file(to, true)?;
	if !SupportedFilesystems::new()?.is_supported(fstype.as_ref()) {
		return Err(Error::new(
			ErrorKind::Unsupported,
			format!("Filesystem \"{}\" is unsupported", fstype.as_ref()),
		));
	}
	Mount::builder()
		.fstype(fstype.as_ref())
		.explicit_loopback()
		.mount(from, to)
}
