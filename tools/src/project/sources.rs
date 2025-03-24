use super::condition::{self, Condition, EvalError};
use super::paths::{self, ProjectPath, ProjectRoot};
use super::{config, generator};
use crate::emit;
use crate::xmlparse::{
    self, attr_pos, elements_children, missing_attribute, node_pos, unexpected_attribute,
    unexpected_root, unexpected_tag,
};
use arcstr::ArcStr;
use roxmltree::{Node, TextPos};
use std::collections::HashSet;
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
// Generators
// ============================================================================

/// A source code generator.
#[derive(Debug)]
pub struct Generator {
    rule: ArcStr,
    name: ArcStr,
    outputs: Vec<Arc<Source>>,
    implementation: Box<dyn generator::Generator>,
}

impl Generator {
    /// Get the name rule that the generator uses for generating outputs.
    pub fn rule(&self) -> &ArcStr {
        &self.rule
    }

    /// Get the name identifying this generator in the spec. This should be
    /// unique among shaders with the same rule.
    pub fn name(&self) -> &ArcStr {
        &self.name
    }

    /// Get all outputs for this generator.
    pub fn outputs(&self) -> &[Arc<Source>] {
        &self.outputs
    }

    /// Get the underlying generator implementation.
    pub fn implementation(&self) -> &dyn generator::Generator {
        self.implementation.as_ref()
    }
}

// ============================================================================
// Source List
// ============================================================================

/// A resolved list of sources and generators for a particular build.
#[derive(Debug)]
pub struct SourceList {
    sources: Vec<Arc<Source>>,
    generators: Vec<Arc<Generator>>,
}

impl SourceList {
    fn new() -> Self {
        Self {
            sources: Vec::new(),
            generators: Vec::new(),
        }
    }

    fn sort(&mut self) {
        self.sources
            .sort_by(|x, y| x.path.as_str().cmp(y.path.as_str()));
        self.generators
            .sort_by(|x, y| x.name.as_str().cmp(y.name.as_str()));
    }

    /// Get all sources in the build.
    pub fn sources(&self) -> &[Arc<Source>] {
        &self.sources
    }

    /// Get all source generators in the build.
    pub fn generators(&self) -> &[Arc<Generator>] {
        &self.generators
    }
}

/// An error running a generator.
#[derive(Debug)]
struct GeneratorRunError {
    rule: ArcStr,
    name: ArcStr,
    err: Box<dyn error::Error>,
}

impl fmt::Display for GeneratorRunError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "generator rule={:?} name={:?} failed: {}",
            self.rule, self.name, self.err
        )
    }
}

impl error::Error for GeneratorRunError {}

/// A set of source code generators.
pub struct GeneratorSet {
    names: HashSet<(ArcStr, ArcStr)>,
    generators: Vec<Arc<Generator>>,
}

impl GeneratorSet {
    /// Create a new, empty set of generators.
    pub fn new() -> Self {
        Self {
            names: HashSet::new(),
            generators: Vec::new(),
        }
    }

    /// Add generators from the given source list.
    pub fn add(&mut self, list: &SourceList) {
        for generator in list.generators.iter() {
            let key = (generator.rule.clone(), generator.name.clone());
            if self.names.insert(key) {
                self.generators.push(generator.clone());
            }
        }
    }

    /// Run all of the code generators.
    pub fn run(
        &self,
        root: &ProjectRoot,
        outputs: &mut emit::Outputs,
    ) -> Result<(), Box<dyn error::Error>> {
        for generator in self.generators.iter() {
            match generator.implementation.run(&root) {
                Ok(files) => {
                    for file in files {
                        outputs.add_file(root.resolve(&file.path), file.data);
                    }
                }
                Err(err) => {
                    return Err(GeneratorRunError {
                        rule: generator.rule.clone(),
                        name: generator.name.clone(),
                        err,
                    }
                    .into());
                }
            }
        }
        Ok(())
    }
}

// ============================================================================
// Error
// ============================================================================

/// Error reading a project spec.
#[derive(Debug)]
pub enum ReadError {
    Condition {
        err: condition::ParseError,
        pos: TextPos,
    },
    BadPath {
        path: String,
        err: paths::PathError,
        pos: TextPos,
    },
    UnknownExtension {
        path: String,
        pos: TextPos,
    },
    IO(io::Error),
    XML(xmlparse::Error),
    Parse(roxmltree::Error),
    Generator {
        err: generator::EvaluationError,
        pos: TextPos,
    },
}

impl From<io::Error> for ReadError {
    fn from(value: io::Error) -> Self {
        ReadError::IO(value)
    }
}

impl From<xmlparse::Error> for ReadError {
    fn from(value: xmlparse::Error) -> Self {
        ReadError::XML(value)
    }
}

impl From<roxmltree::Error> for ReadError {
    fn from(value: roxmltree::Error) -> Self {
        ReadError::Parse(value)
    }
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ReadError::Condition { err, pos } => {
                write!(f, "invalid condition at {}: {}", pos, err)
            }
            ReadError::BadPath { path, err, pos } => {
                write!(f, "invalid path {:?} at {}: {}", path, pos, err)
            }
            ReadError::UnknownExtension { path, pos } => {
                write!(f, "file {:?} at {} has unknown extension", path, pos)
            }
            ReadError::IO(e) => write!(f, "failed to read: {}", e),
            ReadError::XML(err) => err.fmt(f),
            ReadError::Parse(err) => err.fmt(f),
            ReadError::Generator { err, pos } => write!(f, "invalid generator at {}: {}", pos, err),
        }
    }
}

impl error::Error for ReadError {}

// ============================================================================
// Source Spec
// ============================================================================

/// A specification for which sources are included in each build. This directly
/// corresponds to the sources.xml file.
#[derive(Debug)]
pub struct SourceSpec {
    group: Group,
}

impl SourceSpec {
    /// Read the main project source list.
    pub fn read_project(root: &ProjectRoot) -> Result<Self, ReadError> {
        let path = root.resolve_str("support/sources.xml");
        SourceSpec::read(&path)
    }

    /// Read a source list from a file.
    pub fn read(path: &Path) -> Result<Self, ReadError> {
        let text = fs::read_to_string(path)?;
        let doc = roxmltree::Document::parse(&text)?;
        let root = doc.root_element();
        if root.tag_name().name() != "sources" {
            return Err(unexpected_root(root).into());
        }
        Ok(SourceSpec {
            group: Group::parse(root)?,
        })
    }

    /// Return the sources that are included in a specific build configuration.
    pub fn sources_for_config(&self, config: &config::Config) -> Result<SourceList, EvalError> {
        let mut sources = SourceList::new();
        self.group.append_sources_config(&mut sources, config)?;
        sources.sort();
        Ok(sources)
    }

    /// Return all sources.
    pub fn all_sources(&self) -> SourceList {
        let mut sources = SourceList::new();
        self.group.append_sources(&mut sources);
        sources.sort();
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
    generators: Vec<Arc<Generator>>,
    subgroups: Vec<Group>,
}

/// Parse a path attribute.
fn parse_path(
    directory: &ProjectPath,
    node: Node,
    attr: roxmltree::Attribute,
) -> Result<(SourceType, ProjectPath), ReadError> {
    match directory.append(attr.value()) {
        Ok(path) => match path.extension().and_then(SourceType::for_extension) {
            None => Err(ReadError::UnknownExtension {
                path: attr.value().to_string(),
                pos: attr_pos(node, attr),
            }),
            Some(ty) => Ok((ty, path)),
        },
        Err(err) => Err(ReadError::BadPath {
            path: attr.value().into(),
            err,
            pos: attr_pos(node, attr),
        }),
    }
}

/// Parse a <src> tag.
fn parse_source(node: Node) -> Result<Arc<Source>, ReadError> {
    let mut type_path: Option<(SourceType, ProjectPath)> = None;
    for attr in node.attributes() {
        match attr.name() {
            "path" => type_path = Some(parse_path(&ProjectPath::SRC, node, attr)?),
            _ => return Err(unexpected_attribute(node, attr).into()),
        }
    }
    let Some((ty, path)) = type_path else {
        return Err(missing_attribute(node, "path").into());
    };
    Ok(Arc::new(Source { ty, path }))
}

/// Parse an <output> tag.
fn parse_output(node: Node) -> Result<Arc<Source>, ReadError> {
    let mut type_path: Option<(SourceType, ProjectPath)> = None;
    for attr in node.attributes() {
        match attr.name() {
            "path" => type_path = Some(parse_path(&ProjectPath::GENERATED, node, attr)?),
            _ => return Err(unexpected_attribute(node, attr).into()),
        }
    }
    let Some((ty, path)) = type_path else {
        return Err(missing_attribute(node, "path").into());
    };
    Ok(Arc::new(Source { ty, path }))
}

/// Parse a <generator> tag.
fn parse_generator(node: Node) -> Result<Arc<Generator>, ReadError> {
    let mut rule: Option<&str> = None;
    let mut name: Option<&str> = None;
    for attr in node.attributes() {
        match attr.name() {
            "rule" => rule = Some(attr.value()),
            "name" => name = Some(attr.value()),
            _ => return Err(unexpected_attribute(node, attr).into()),
        }
    }
    let Some(rule) = rule else {
        return Err(missing_attribute(node, "rule").into());
    };
    let Some(name) = name else {
        return Err(missing_attribute(node, "name").into());
    };
    let mut outputs: Vec<Arc<Source>> = Vec::new();
    for child in elements_children(node) {
        let child = child?;
        match child.tag_name().name() {
            "output" => outputs.push(parse_output(child)?),
            _ => return Err(unexpected_tag(child, node).into()),
        }
    }
    let implementation = match generator::evaluate(rule, &outputs) {
        Ok(value) => value,
        Err(err) => {
            return Err(ReadError::Generator {
                err,
                pos: node_pos(node),
            });
        }
    };
    Ok(Arc::new(Generator {
        rule: rule.into(),
        name: name.into(),
        outputs,
        implementation,
    }))
}

impl Group {
    /// Parse a group in an XML document.
    fn parse(node: Node) -> Result<Self, ReadError> {
        let mut result = Group {
            condition: None,
            sources: Vec::new(),
            generators: Vec::new(),
            subgroups: Vec::new(),
        };
        for attr in node.attributes() {
            match attr.name() {
                "condition" => match Condition::parse(attr.value().as_bytes()) {
                    Ok(condition) => result.condition = Some(condition),
                    Err(err) => {
                        return Err(ReadError::Condition {
                            err,
                            pos: attr_pos(node, attr),
                        });
                    }
                },
                _ => return Err(unexpected_attribute(node, attr).into()),
            }
        }
        for child in elements_children(node) {
            let child = child?;
            match child.tag_name().name() {
                "group" => result.subgroups.push(Group::parse(child)?),
                "src" => result.sources.push(parse_source(child)?),
                "generator" => {
                    let generator = parse_generator(child)?;
                    result.sources.extend_from_slice(&generator.outputs);
                    result.generators.push(generator);
                }
                _ => return Err(unexpected_tag(child, node).into()),
            }
        }
        Ok(result)
    }

    fn append_self(&self, out: &mut SourceList) {
        out.sources.extend_from_slice(&self.sources);
        out.generators.extend_from_slice(&self.generators);
    }

    fn append_sources(&self, out: &mut SourceList) {
        self.append_self(out);
        for group in self.subgroups.iter() {
            group.append_sources(out);
        }
    }

    fn append_sources_config(
        &self,
        out: &mut SourceList,
        config: &config::Config,
    ) -> Result<(), EvalError> {
        if let Some(condition) = &self.condition {
            if !condition.evaluate(|tag| config.eval_tag(tag))? {
                return Ok(());
            }
        }
        self.append_self(out);
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
