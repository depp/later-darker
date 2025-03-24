use super::condition::{self, Condition, EvalError};
use super::config;
use super::paths::{self, ProjectPath, ProjectRoot};
use crate::xmlparse::{self, attr_pos, unexpected_attribute, unexpected_root, unexpected_tag};
use roxmltree::{Node, NodeType, TextPos};
use std::error;
use std::fmt;
use std::fs;
use std::io;
use std::path::Path;
use std::sync::Arc;

// ============================================================================
// Source Types
// ============================================================================

const SOURCE_EXTENSION: &str = "cpp";
const HEADER_EXTENSION: &str = "hpp";

/// A type of source file.
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
}

// ============================================================================
// Source Files
// ============================================================================

/// Information about an individual source file in the project.
#[derive(Debug, Clone)]
pub struct Source {
    ty: SourceType,
    path: ProjectPath,
}

impl Source {
    /// Create a new generated source file.
    pub fn new_generated(name: &str, ty: SourceType) -> Result<Arc<Self>, paths::PathError> {
        let full_name = [name, ty.extension()].join(".");
        let path = paths::ProjectPath::GENERATED.append(&full_name)?;
        Ok(Arc::new(Source { ty, path }))
    }

    /// Get the source type.
    pub fn ty(&self) -> SourceType {
        self.ty
    }

    /// Get the Unix-style, relative path for this source.
    pub fn path(&self) -> &ProjectPath {
        &self.path
    }
}

// ============================================================================
// Error
// ============================================================================

/// Error reading a project spec.
#[derive(Debug)]
pub enum InnerReadError {
    IO(io::Error),
    BuildTag(condition::ParseError, TextPos),
    BadPath(String, paths::PathError),
    UnknownExtension(String),
}

impl InnerReadError {
    fn err(self) -> ReadError {
        ReadError::Other(self)
    }
}

impl fmt::Display for InnerReadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InnerReadError::IO(e) => write!(f, "failed to read: {}", e),
            InnerReadError::BuildTag(err, pos) => {
                write!(f, "invalid condition at {}: {}", pos, err)
            }
            InnerReadError::BadPath(path, err) => write!(f, "invalid path {:?}: {}", path, err),
            InnerReadError::UnknownExtension(path) => {
                write!(f, "file {:?} has unknown extension", path)
            }
        }
    }
}

impl error::Error for InnerReadError {}

/// Error from reading a source list.
pub type ReadError = xmlparse::Error<InnerReadError>;

// ============================================================================
// Source List
// ============================================================================

/// A list of source files.
#[derive(Debug)]
pub struct SourceList {
    group: Group,
}

/// Sort a list of sources lexicographically.
fn sort_sources(sources: &mut [Arc<Source>]) {
    sources.sort_by(|x, y| x.path.as_str().cmp(y.path.as_str()));
}

impl SourceList {
    /// Read the main project source list.
    pub fn read_project(root: &ProjectRoot) -> Result<Self, ReadError> {
        let path = root.resolve_str("support/sources.xml");
        SourceList::read(&path)
    }

    /// Read a source list from a file.
    pub fn read(path: &Path) -> Result<Self, ReadError> {
        let text =
            fs::read_to_string(path).map_err(|err| ReadError::Other(InnerReadError::IO(err)))?;
        let doc = roxmltree::Document::parse(&text)?;
        let root = doc.root_element();
        if root.tag_name().name() != "sources" {
            return Err(unexpected_root(root).into());
        }
        Ok(SourceList {
            group: Group::parse(root)?,
        })
    }

    /// Return the sources that are included in a specific build configuration.
    pub fn sources_for_config(
        &self,
        config: &config::Config,
    ) -> Result<Vec<Arc<Source>>, EvalError> {
        let mut sources = Vec::new();
        self.group.append_sources_config(&mut sources, config)?;
        sort_sources(&mut sources);
        Ok(sources)
    }

    /// Return all sources.
    pub fn all_sources(&self) -> Vec<Arc<Source>> {
        let mut sources = Vec::new();
        self.group.append_sources(&mut sources);
        sort_sources(&mut sources);
        sources
    }

    /// Count the number of sources in the list.
    pub fn count(&self) -> usize {
        self.group.count()
    }
}

/// A group of sources in the source list, which can contain subgroups.
#[derive(Debug)]
struct Group {
    condition: Option<Condition>,
    sources: Vec<Arc<Source>>,
    subgroups: Vec<Group>,
}

impl Group {
    /// Parse a group in an XML document.
    fn parse(node: Node) -> Result<Self, ReadError> {
        let mut result = Group {
            condition: None,
            sources: Vec::new(),
            subgroups: Vec::new(),
        };
        for attr in node.attributes() {
            match attr.name() {
                "condition" => match Condition::parse(attr.value().as_bytes()) {
                    Ok(condition) => result.condition = Some(condition),
                    Err(err) => {
                        return Err(InnerReadError::BuildTag(err, attr_pos(node, attr)).err());
                    }
                },
                _ => return Err(unexpected_attribute(node, attr)),
            }
        }
        // Combine all text and parse it once combined, in case adjacent text
        // nodes are not combined.
        let mut text = String::new();
        for child in node.children() {
            match child.node_type() {
                NodeType::Element => {
                    text.push(' ');
                    match child.tag_name().name() {
                        "group" => result.subgroups.push(Group::parse(child)?),
                        _ => return Err(unexpected_tag(child, node)),
                    }
                }
                NodeType::Text => {
                    if let Some(node_text) = child.text() {
                        text.push_str(node_text);
                    }
                }
                _ => (),
            }
        }
        for item in text.split_ascii_whitespace() {
            let path = match ProjectPath::SRC.append(item) {
                Ok(path) => path,
                Err(err) => return Err(InnerReadError::BadPath(item.into(), err).err()),
            };
            let Some(ty) = path.extension().and_then(SourceType::for_extension) else {
                return Err(InnerReadError::UnknownExtension(item.into()).err());
            };
            result.sources.push(Arc::new(Source { path, ty }));
        }
        Ok(result)
    }

    fn append_sources(&self, out: &mut Vec<Arc<Source>>) {
        out.extend_from_slice(&self.sources);
        for group in self.subgroups.iter() {
            group.append_sources(out);
        }
    }

    fn append_sources_config(
        &self,
        out: &mut Vec<Arc<Source>>,
        config: &config::Config,
    ) -> Result<(), EvalError> {
        if let Some(condition) = &self.condition {
            if !condition.evaluate(|tag| config.eval_tag(tag))? {
                return Ok(());
            }
        }
        out.extend_from_slice(&self.sources);
        for group in self.subgroups.iter() {
            group.append_sources_config(out, config)?;
        }
        Ok(())
    }

    fn count(&self) -> usize {
        self.sources.len()
            + self
                .subgroups
                .iter()
                .map(|group| group.count())
                .sum::<usize>()
    }
}
