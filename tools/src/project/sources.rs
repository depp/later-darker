use super::buildtag;
use super::config;
use super::paths;
use arcstr::ArcStr;
use std::error;
use std::ffi::OsStr;
use std::fmt;
use std::fs;
use std::io;
use std::io::BufRead;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Debug, Clone, Copy)]
pub enum SourceType {
    Source,
    Header,
}

impl SourceType {
    fn for_extension(ext: &OsStr) -> Option<Self> {
        match ext.to_str()? {
            "cpp" => Some(SourceType::Source),
            "hpp" => Some(SourceType::Header),
            _ => None,
        }
    }
}

/// Information about an individual source file.
#[derive(Debug, Clone)]
pub struct Source {
    pub ty: SourceType,

    /// Unix-style path.
    pub path: ArcStr,

    /// Build tag, if present.
    pub build_tag: Option<Arc<buildtag::Expression>>,
}

#[derive(Debug)]
pub enum ScanError {
    IO(PathBuf, io::Error),
    BuildTag(PathBuf, BuildTagError),
}

impl fmt::Display for ScanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScanError::IO(p, e) => write!(f, "{}: list files: {}", p.display(), e),
            ScanError::BuildTag(p, e) => write!(f, "{}: get build tag: {}", p.display(), e),
        }
    }
}

impl error::Error for ScanError {}

/// A list of source files.
#[derive(Debug, Clone)]
pub struct SourceList {
    pub sources: Vec<Source>,
}

/// Get the build tag associated with a file.
fn build_tag_from_filename(name: &str) -> Option<&str> {
    let (stem, _) = name.rsplit_once('.')?;
    let (_, tag) = stem.rsplit_once('_')?;
    if config::is_tag(tag) { Some(tag) } else { None }
}

impl SourceList {
    /// Scan the project root directory for source files.
    pub fn scan(directory: &Path) -> Result<Self, ScanError> {
        let src = directory.join(paths::SRC);
        let mut sources: Vec<Source> = Vec::new();
        let items = match fs::read_dir(&src) {
            Ok(items) => items,
            Err(e) => return Err(ScanError::IO(src, e)),
        };
        for item in items {
            let item = match item {
                Ok(item) => item,
                Err(e) => return Err(ScanError::IO(src, e)),
            };
            let name = PathBuf::from(item.file_name());
            let Some(ty) = name.extension().and_then(SourceType::for_extension) else {
                continue;
            };
            let name = match name.into_os_string().into_string() {
                Ok(name) => name,
                Err(name) => {
                    eprintln!("Warning: invalid filename: {:?}", name);
                    continue;
                }
            };
            if name.starts_with('.') || name.starts_with('_') {
                continue;
            }
            let path = src.join(&name);
            let build_tag = match read_build_tag(&path) {
                Ok(e) => e,
                Err(e) => return Err(ScanError::BuildTag(path, e)),
            };
            let build_tag = match build_tag {
                None => match build_tag_from_filename(&name) {
                    None => None,
                    Some(tag) => Some(buildtag::Expression::tag(tag.into())),
                },
                Some(value) => Some(value),
            };
            sources.push(Source {
                ty,
                path: format!("{}/{}", paths::SRC, name).into(),
                build_tag: build_tag.map(Arc::new),
            });
        }
        sources.sort_by(|x, y| x.path.cmp(&y.path));
        Ok(SourceList { sources })
    }
}

/// Error from reading a build tag.
#[derive(Debug)]
pub enum BuildTagError {
    IO(io::Error),
    Parse(u32, buildtag::ParseError),
}

impl From<io::Error> for BuildTagError {
    fn from(value: io::Error) -> Self {
        BuildTagError::IO(value)
    }
}

impl fmt::Display for BuildTagError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BuildTagError::IO(e) => write!(f, "read: {}", e),
            BuildTagError::Parse(lineno, e) => write!(f, "line {}: {}", lineno, e),
        }
    }
}

impl error::Error for BuildTagError {}

/// Read the build tag for a single file.
fn read_build_tag(path: &Path) -> Result<Option<buildtag::Expression>, BuildTagError> {
    let file = fs::File::open(path)?;
    let mut reader = io::BufReader::new(file);
    let mut line = Vec::new();
    let mut lineno: u32 = 0;
    loop {
        line.clear();
        let n = reader.read_until(b'\n', &mut line)?;
        if n == 0 {
            break;
        }
        lineno += 1;
        let line = line.trim_ascii_start();
        if line.starts_with(b"//") {
            const PREFIX: &[u8] = b"//build:";
            if line.starts_with(PREFIX) {
                let line = line[PREFIX.len()..].trim_ascii();
                return match buildtag::Expression::parse(line) {
                    Ok(e) => Ok(Some(e)),
                    Err(e) => Err(BuildTagError::Parse(lineno, e)),
                };
            }
        } else if !line.is_empty() {
            break;
        }
    }
    Ok(None)
}
