use super::paths;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

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
    pub path: String,
}

/// Scan the project root directory for source files.
pub fn scan(directory: &Path) -> Result<Vec<Source>, io::Error> {
    let src = directory.join(paths::SRC);
    let mut sources: Vec<Source> = Vec::new();
    for item in fs::read_dir(&src)? {
        let item = item?;
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
        sources.push(Source {
            ty,
            path: format!("{}/{}", paths::SRC, name),
        });
    }
    sources.sort_by(|x, y| x.path.cmp(&y.path));
    Ok(sources)
}
