use super::condition;
use super::condition::Condition;
use super::config;
use super::paths;
use super::paths::{ProjectPath, ProjectRoot};
use std::error;
use std::fmt;
use std::fs;
use std::io;
use std::io::BufRead;
use std::path::Path;
use std::sync::Arc;

const SOURCE_EXTENSION: &str = "cpp";
const HEADER_EXTENSION: &str = "hpp";

#[derive(Debug, Clone, Copy)]
pub enum SourceType {
    Source,
    Header,
}

impl SourceType {
    fn extension(&self) -> &'static str {
        match self {
            SourceType::Source => SOURCE_EXTENSION,
            SourceType::Header => HEADER_EXTENSION,
        }
    }

    fn for_extension(ext: &str) -> Option<Self> {
        Some(match ext {
            SOURCE_EXTENSION => SourceType::Source,
            HEADER_EXTENSION => SourceType::Header,
            _ => return None,
        })
    }

    fn for_filename(name: &str) -> Option<Self> {
        let (_, extension) = name.rsplit_once('.')?;
        Self::for_extension(extension)
    }
}

/// Information about an individual source file in the project.
#[derive(Debug, Clone)]
pub struct Source {
    ty: SourceType,

    path: ProjectPath,

    condition: Condition,
}

impl Source {
    /// Create a new generated source file.
    pub fn new_generated(name: &str, ty: SourceType) -> Result<Arc<Self>, paths::PathError> {
        let full_name = [name, ty.extension()].join(".");
        let path = paths::ProjectPath::GENERATED.append(&full_name)?;
        Ok(Arc::new(Source {
            ty,
            path,
            condition: Condition::Value(true),
        }))
    }

    /// Get the source type.
    pub fn ty(&self) -> SourceType {
        self.ty
    }

    /// Get the Unix-style, relative path for this source.
    pub fn path(&self) -> &ProjectPath {
        &self.path
    }

    /// Test whether this source is included in the given config.
    pub fn is_in_config(&self, config: &config::Config) -> Result<bool, condition::EvalError> {
        self.condition.evaluate(|tag| config.eval_tag(tag))
    }

    /// Get the build tag for this source file.
    pub fn condition(&self) -> &Condition {
        &self.condition
    }
}

#[derive(Debug)]
pub enum ScanError {
    IO(ProjectPath, io::Error),
    BuildTag(ProjectPath, BuildTagError),
}

impl fmt::Display for ScanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScanError::IO(p, e) => write!(f, "{}: list files: {}", p, e),
            ScanError::BuildTag(p, e) => write!(f, "{}: get build tag: {}", p, e),
        }
    }
}

impl error::Error for ScanError {}

#[derive(Debug, Clone)]
pub struct FilterError(ProjectPath, pub condition::EvalError);

impl fmt::Display for FilterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.0, self.1)
    }
}

impl error::Error for FilterError {}

/// A list of source files.
#[derive(Debug, Clone)]
pub struct SourceList {
    pub sources: Vec<Arc<Source>>,
}

/// Get the build tag associated with a file.
fn build_tag_from_filename(name: &str) -> Option<&str> {
    let (stem, _) = name.rsplit_once('.')?;
    let (_, tag) = stem.rsplit_once('_')?;
    if config::is_tag(tag) { Some(tag) } else { None }
}

impl SourceList {
    /// Scan the project root directory for source files.
    pub fn scan(project_root: &ProjectRoot) -> Result<Self, ScanError> {
        let directory = &ProjectPath::SRC;
        let mut sources: Vec<Arc<Source>> = Vec::new();
        let items = match fs::read_dir(&project_root.resolve(directory)) {
            Ok(items) => items,
            Err(e) => return Err(ScanError::IO(ProjectPath::SRC.clone(), e)),
        };
        for item in items {
            let item = match item {
                Ok(item) => item,
                Err(e) => return Err(ScanError::IO(ProjectPath::SRC.clone(), e)),
            };
            let name = match item.file_name().into_string() {
                Ok(name) => name,
                Err(name) => {
                    eprintln!("Warning: bad filename encoding: {:?}", name);
                    continue;
                }
            };
            if name.starts_with('.') || name.starts_with('_') {
                continue;
            }
            let Some(ty) = SourceType::for_filename(&name) else {
                continue;
            };
            let path = match directory.append(&name) {
                Ok(path) => path,
                Err(e) => {
                    eprintln!("Warning: bad filename {:?}: {}", name, e);
                    continue;
                }
            };
            let condition = match read_build_tag(&project_root.resolve(&path)) {
                Ok(e) => e,
                Err(e) => return Err(ScanError::BuildTag(path, e)),
            };
            let condition = match condition {
                None => match build_tag_from_filename(&name) {
                    None => Condition::Value(true),
                    Some(tag) => Condition::Atom(tag.into()),
                },
                Some(value) => value,
            };
            sources.push(Arc::new(Source {
                ty,
                path,
                condition,
            }));
        }
        Ok(SourceList { sources })
    }

    /// Filter sources that apply to a build configuration.
    pub fn filter(&self, config: &config::Config) -> Result<Self, FilterError> {
        let mut sources = Vec::with_capacity(self.sources.len());
        for src in self.sources.iter() {
            match src.is_in_config(config) {
                Ok(value) => {
                    if value {
                        sources.push(src.clone());
                    }
                }
                Err(e) => return Err(FilterError(src.path.clone(), e)),
            }
        }
        Ok(SourceList { sources })
    }

    /// Sort the sources by path.
    pub fn sort(&mut self) {
        self.sources
            .sort_by(|x, y| x.path.as_str().cmp(y.path.as_str()));
    }
}

/// Error from reading a build tag.
#[derive(Debug)]
pub enum BuildTagError {
    IO(io::Error),
    Parse(u32, condition::ParseError),
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
fn read_build_tag(path: &Path) -> Result<Option<condition::Condition>, BuildTagError> {
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
                return match condition::Condition::parse(line) {
                    Ok(condition) => Ok(Some(condition)),
                    Err(e) => Err(BuildTagError::Parse(lineno, e)),
                };
            }
        } else if !line.is_empty() {
            break;
        }
    }
    Ok(None)
}
