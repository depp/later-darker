use arcstr::{ArcStr, literal};
use std::env;
use std::error;
use std::ffi::OsString;
use std::fmt;
use std::path::{MAIN_SEPARATOR_STR, Path, PathBuf};

/// Error that indicates that the project root directory could not be found.
#[derive(Debug, Clone, Copy)]
pub struct NoProjectDirectory;

impl fmt::Display for NoProjectDirectory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("no project directory specified")
    }
}

impl error::Error for NoProjectDirectory {}

/// The project root directory.
#[derive(Debug)]
pub struct ProjectRoot(ProjectPath);

impl ProjectRoot {
    fn from_path(path: PathBuf) -> Self {
        ProjectRoot(ProjectPath {
            full: path.into_os_string(),
            relative: literal!("."),
        })
    }

    /// Find the project root directory.
    pub fn find() -> Result<Self, NoProjectDirectory> {
        let Some(manifest_dir) = env::var_os("CARGO_MANIFEST_DIR") else {
            return Err(NoProjectDirectory);
        };
        let mut dir = PathBuf::from(manifest_dir);
        dir.pop();
        Ok(Self::from_path(dir))
    }

    /// Find the project root directory, or take it form a command-line flag.
    pub fn find_or(project_directory: Option<&Path>) -> Result<Self, NoProjectDirectory> {
        match project_directory {
            None => Self::find(),
            Some(value) => Ok(Self::from_path(value.to_path_buf())),
        }
    }

    /// Get the root.
    pub fn root(&self) -> &ProjectPath {
        &self.0
    }

    /// Get the source directory.
    pub fn src(&self) -> ProjectPath {
        self.0.join("src").unwrap()
    }

    /// Get the generated source directory.
    pub fn generated(&self) -> ProjectPath {
        self.0.join("src/generated").unwrap()
    }
}

/// Error that indicates an invalid path.
#[derive(Debug)]
pub struct InvalidPathError(String);

impl fmt::Display for InvalidPathError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid path: {:?}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct ProjectPath {
    full: OsString,
    relative: ArcStr,
}

impl ProjectPath {
    /// Get the full path.
    pub fn full(&self) -> &Path {
        Path::new(&self.full)
    }

    /// Get the Unix relative path.
    pub fn unix(&self) -> &ArcStr {
        &self.relative
    }

    /// Create the Windows relative path.
    pub fn windows(&self) -> String {
        self.relative.replace('/', "\\")
    }

    /// Join a path to this one.
    pub fn join(&self, name: &str) -> Result<ProjectPath, InvalidPathError> {
        if name.bytes().any(|c| !c.is_ascii_graphic() || c == b'\\') {
            return Err(InvalidPathError(name.to_string()));
        }
        let mut full = self.full.clone();
        let mut relative = self.relative.to_string();
        if name != "." && !name.is_empty() {
            for part in name.split('/') {
                if part.starts_with(".") || part.is_empty() {
                    return Err(InvalidPathError(name.to_string()));
                }
                full.push(MAIN_SEPARATOR_STR);
                full.push(part);
                if relative == "." || relative.is_empty() {
                    relative.clear();
                } else {
                    relative.push('/');
                }
                relative.push_str(part)
            }
        }
        Ok(ProjectPath {
            full,
            relative: ArcStr::from(relative),
        })
    }
}

impl fmt::Display for ProjectPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.full().display().fmt(f)
    }
}
