use std::env;
use std::error;
use std::fmt;
use std::path::Path;
use std::path::PathBuf;

/// Error that indicates that the project root directory could not be found.
#[derive(Debug, Clone, Copy)]
pub struct NoProjectDirectory;

impl fmt::Display for NoProjectDirectory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("no project directory specified")
    }
}

impl error::Error for NoProjectDirectory {}

/// Find the project root directory.
pub fn find_project_directory() -> Result<PathBuf, NoProjectDirectory> {
    let Some(manifest_dir) = env::var_os("CARGO_MANIFEST_DIR") else {
        return Err(NoProjectDirectory);
    };
    let mut dir = PathBuf::from(manifest_dir);
    dir.pop();
    Ok(dir)
}

/// Find the project root directory, or take it form a command-line flag.
pub fn find_project_directory_or(
    project_directory: Option<&Path>,
) -> Result<PathBuf, NoProjectDirectory> {
    match project_directory {
        None => find_project_directory(),
        Some(value) => Ok(value.to_path_buf()),
    }
}

pub const SRC: &str = "src";
