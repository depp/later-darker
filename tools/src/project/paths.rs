use arcstr::{ArcStr, literal};
use std::env;
use std::error;
use std::ffi::OsStr;
use std::fmt;
use std::path::{MAIN_SEPARATOR, Path, PathBuf};

// ============================================================================
// Errors
// ============================================================================

/// Error that indicates that the project root directory could not be found.
#[derive(Debug)]
pub struct NoProjectDirectory;

impl fmt::Display for NoProjectDirectory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("no project directory specified")
    }
}

impl error::Error for NoProjectDirectory {}

/// Error that indicates a path is invalid.
#[derive(Debug)]
pub enum PathError {
    LeadingSlash,
    TrailingSlash,
    EmptyComponent,
    InvalidComponent(&'static str),
    InvalidCharacter(char),
    Reserved(ArcStr),
}

impl fmt::Display for PathError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PathError::LeadingSlash => f.write_str("path has leading slash"),
            PathError::TrailingSlash => f.write_str("path has trailing slash"),
            PathError::EmptyComponent => f.write_str("path has empty component"),
            PathError::InvalidComponent(name) => {
                write!(f, "path contains invalid component: {:?}", name)
            }
            PathError::InvalidCharacter(c) => write!(f, "path contains invalid character: {:?}", c),
            PathError::Reserved(name) => write!(f, "path contains reserved name: {:?}", name),
        }
    }
}

impl error::Error for PathError {}

// ============================================================================
// Project Root
// ============================================================================

/// The project root directory.
#[derive(Debug)]
pub struct ProjectRoot(PathBuf);

impl ProjectRoot {
    /// Find the project root directory.
    pub fn find() -> Result<Self, NoProjectDirectory> {
        let Some(manifest_dir) = env::var_os("CARGO_MANIFEST_DIR") else {
            return Err(NoProjectDirectory);
        };
        let mut dir = PathBuf::from(manifest_dir);
        dir.pop();
        Ok(Self(dir))
    }

    /// Find the project root directory, or take it form a command-line flag.
    pub fn find_or(project_directory: Option<&Path>) -> Result<Self, NoProjectDirectory> {
        match project_directory {
            None => Self::find(),
            Some(value) => Ok(Self(value.to_path_buf())),
        }
    }

    /// Resolve a project path.
    pub fn resolve(&self, path: &ProjectPath) -> PathBuf {
        let mut buf = self.0.clone().into_os_string();
        let path = path.0.as_str();
        if path != "." {
            if MAIN_SEPARATOR == '/' {
                if !buf.as_encoded_bytes().ends_with(b"/") {
                    buf.push(OsStr::new("/"));
                }
                buf.push(OsStr::new(path));
            } else {
                // There are only two separators: forward and backward slash.
                if !buf.as_encoded_bytes().ends_with(b"\\") {
                    buf.push(OsStr::new("\\"));
                }
                for (n, part) in path.split('/').enumerate() {
                    if n != 0 {
                        buf.push(OsStr::new("\\"));
                    }
                    buf.push(OsStr::new(part));
                }
            }
        }
        buf.into()
    }

    /// Get the root as a path.
    pub fn as_path(&self) -> &Path {
        Path::new(&self.0)
    }
}

// ============================================================================
// Project Path
// ============================================================================

/// Return true if the name is a reserved name on Windows.
fn is_reserved_name(name: &str) -> bool {
    let name = name.as_bytes();
    match name.len() {
        3 => {
            let mut arr: [u8; 3] = name.try_into().unwrap();
            for c in arr.iter_mut() {
                *c = c.to_ascii_lowercase();
            }
            match &arr[..] {
                b"con" | b"prn" | b"aux" | b"nul" | b"com" | b"lpt" => true,
                _ => false,
            }
        }
        4 => {
            if name[3].is_ascii_digit() {
                let mut arr: [u8; 3] = name.try_into().unwrap();
                for c in arr.iter_mut() {
                    *c = c.to_ascii_lowercase();
                }
                match &arr[..] {
                    b"com" | b"lpt" => true,
                    _ => false,
                }
            } else {
                false
            }
        }
        _ => false,
    }
}

/// Validate that a path component is safe to use in a project.
fn validate_component(component: &str) -> Result<(), PathError> {
    // https://learn.microsoft.com/en-us/windows/win32/fileio/naming-a-file
    if component.is_empty() {
        return Err(PathError::EmptyComponent);
    }
    if component.starts_with('.') {
        match component {
            "." => return Err(PathError::InvalidComponent(".")),
            ".." => return Err(PathError::InvalidComponent("..")),
            _ => (),
        }
    }
    for c in component.chars() {
        if '\x21' <= c && c <= '\x7e' {
            match c {
                '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => {
                    return Err(PathError::InvalidCharacter(c));
                }
                _ => (),
            }
        } else {
            return Err(PathError::InvalidCharacter(c));
        }
    }
    let stem = match component.split_once('.') {
        None => component,
        Some((stem, _)) => stem,
    };
    if is_reserved_name(stem) {
        return Err(PathError::Reserved(ArcStr::from(stem)));
    }
    Ok(())
}

/// Validate that a path is safe to use in a project.
fn validate_path(path: &str) -> Result<(), PathError> {
    if path.starts_with("/") {
        return Err(PathError::LeadingSlash);
    }
    if path.ends_with("/") {
        return Err(PathError::LeadingSlash);
    }
    for component in path.split('/') {
        validate_component(component)?;
    }
    Ok(())
}

/// A path to a file or directory within the project. Paths are validated.
#[derive(Debug, Clone)]
pub struct ProjectPath(ArcStr);

impl ProjectPath {
    /// The top-level project directory.
    pub const ROOT: Self = ProjectPath(literal!("."));

    /// The src directory.
    pub const SRC: Self = ProjectPath(literal!("src"));

    /// The generated sources directory.
    pub const GENERATED: Self = ProjectPath(literal!("src/generated"));

    /// Construct a new path by appending a single component to this one.
    pub fn append(&self, name: &str) -> Result<Self, PathError> {
        validate_component(name)?;
        Ok(Self(if self.0 == "." {
            ArcStr::from(name)
        } else {
            let size = self.0.len() + 1 + name.len();
            let mut data = String::with_capacity(size);
            data.push_str(&self.0);
            data.push('/');
            data.push_str(name);
            ArcStr::from(data)
        }))
    }

    /// Construct a new path.
    pub fn new(&self, name: &str) -> Result<Self, PathError> {
        Ok(if name == "." {
            ProjectPath::ROOT
        } else {
            validate_path(name)?;
            Self(ArcStr::from(name))
        })
    }

    /// Get the path as a string.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Create a Windows version of the path.
    pub fn to_windows(&self) -> String {
        self.0.replace('/', "\\")
    }
}

impl fmt::Display for ProjectPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 == "." {
            f.write_str("$project")
        } else {
            write!(f, "$project/{}", self.0)
        }
    }
}

impl AsRef<str> for ProjectPath {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl AsRef<ArcStr> for ProjectPath {
    fn as_ref(&self) -> &ArcStr {
        &self.0
    }
}
