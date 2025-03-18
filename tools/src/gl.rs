use khronos_api;
use roxmltree::{self, Document, Node, NodeType};
use std::collections::{HashMap, HashSet};
use std::error;
use std::fmt::{self, Write as _};
use std::ops::Range;
use std::str;

const APIENTRY: &str = "GLAPIENTRY";
const LINKABLE_VERSION: Version = Version(1, 1);
const MAX_VERSION: Version = Version(3, 3);

/// A type definition in the OpenGL API.
#[derive(Debug, Clone)]
struct Type {
    name: Option<String>,
    definition: String,
}

#[derive(Debug, Clone)]
pub enum GenerateError {
    UnexpectedTag(String),
    MissingCommandProto,
    MissingCommandName,
    DuplicateCommand(String),
    MissingCommand(String),
    MissingAttribute(&'static str),
    InvalidVersion(String),
    InvalidRemoveProfile,
    RemoveMissing(String),
    DuplicateEnum(String),
}

impl fmt::Display for GenerateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GenerateError::UnexpectedTag(tag) => write!(f, "unexpected tag: <{}>", tag),
            GenerateError::MissingCommandProto => f.write_str("missing command <proto>"),
            GenerateError::MissingCommandName => f.write_str("missing command <name>"),
            GenerateError::DuplicateCommand(name) => write!(f, "duplicate command: {:?}", name),
            GenerateError::MissingCommand(name) => {
                write!(f, "could not find command definition: {:?}", name)
            }
            GenerateError::MissingAttribute(name) => {
                write!(f, "missing required attribute: {}", name)
            }
            GenerateError::InvalidVersion(text) => write!(f, "invalid version number: {:?}", text),
            GenerateError::InvalidRemoveProfile => write!(f, "invalid profile for remove"),
            GenerateError::RemoveMissing(name) => {
                write!(f, "cannot remove unknown item: {:?}", name)
            }
            GenerateError::DuplicateEnum(name) => write!(f, "duplicate enum: {:?}", name),
        }
    }
}

impl error::Error for GenerateError {}

type GenerateErrorRange<'input> = (GenerateError, Option<(&'input str, Range<usize>)>);

fn unexpected_tag<'a, 'input>(
    parent: Node<'a, 'input>,
    child: Node<'a, 'input>,
) -> GenerateErrorRange<'input> {
    (
        GenerateError::UnexpectedTag(child.tag_name().name().to_string()),
        Some((parent.tag_name().name(), child.range())),
    )
}

#[derive(Debug)]
pub struct GenerateErrorPos {
    pub error: GenerateError,
    pub pos: Option<(String, roxmltree::TextPos)>,
}

impl fmt::Display for GenerateErrorPos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.pos {
            None => self.error.fmt(f),
            Some((tag, pos)) => write!(f, "line {}: in <{}>: {}", pos.row, tag, self.error),
        }
    }
}

#[derive(Debug)]
pub enum Error {
    XML(roxmltree::Error),
    Generate(GenerateErrorPos),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::XML(e) => write!(f, "could not parse OpenGL spec: {}", e),
            Error::Generate(e) => write!(f, "could not generate OpenGL API: {}", e),
        }
    }
}

impl error::Error for Error {}

fn require_attribute<'a, 'input>(
    node: Node<'a, 'input>,
    name: &'static str,
) -> Result<&'a str, GenerateErrorRange<'input>> {
    match node.attribute(name) {
        None => Err((
            GenerateError::MissingAttribute(name),
            Some((node.tag_name().name(), node.range())),
        )),
        Some(text) => Ok(text),
    }
}

/// Parse an element which only contains text. Return the text.
fn parse_text_contents<'input>(
    node: Node<'_, 'input>,
) -> Result<String, GenerateErrorRange<'input>> {
    let mut result = String::new();
    for child in node.children() {
        match child.node_type() {
            NodeType::Text => {
                if let Some(text) = child.text() {
                    result.push_str(text);
                }
            }
            NodeType::Element => {
                return Err((
                    GenerateError::UnexpectedTag(child.tag_name().name().to_string()),
                    Some((node.tag_name().name(), child.range())),
                ));
            }
            _ => (),
        }
    }
    Ok(result)
}

/// Parse a <type> tag.
/*
fn parse_type(node: Node) -> Result<Type, GenerateError> {
    assert_eq!(node.tag_name().name(), "type");
    let mut name: Option<String> = None;
    let mut definition = String::new();
    for child in node.children() {
        match child.node_type() {
            NodeType::Text => {
                if let Some(text) = child.text() {
                    definition.push_str(text);
                }
            }
            NodeType::Element => match child.tag_name().name() {
                "apientry" => definition.push_str(APIENTRY),
                "name" => {
                    let text = parse_text_contents(child)?;
                    definition.push_str(&text);
                    name = Some(text);
                }
                other => eprintln!("Unknown tag in <type>: <{}>", other),
            },
            _ => (),
        }
    }
    Ok(Type { name, definition })
}

/// Parse a <types> tag.
fn parse_types(node: Node, types: &mut Vec<Type>) -> Result<(), GenerateError> {
    assert_eq!(node.tag_name().name(), "types");
    for child in node.children() {
        if child.is_element() {
            match child.tag_name().name() {
                "type" => types.push(parse_type(child)?),
                other => eprintln!("Unknown tag in <types>: <{}>", other),
            }
        }
    }
    Ok(())
}

/// Parse a <registry> tag.
fn parse_registry(node: Node) -> Result<(), GenerateError> {
    assert_eq!(node.tag_name().name(), "registry");
    let mut types: Vec<Type> = Vec::new();
    for child in node.children() {
        if child.is_element() {
            match child.tag_name().name() {
                "types" => parse_types(child, &mut types)?,
                "comment" => (),
                "feature" => (),
                "enums" => (),
                "commands" => (),
                "groups" => (),
                "extensions" => (),
                other => {
                    eprintln!("Unknown tag in <registry>: <{}>", other);
                }
            }
        }
    }
    for ty in types.iter() {
        eprintln!("{:?}", ty);
    }
    Ok(())
}
    */
// ============================================================================
// Feature & Version Map
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Version(u8, u8);

impl Version {
    fn parse(text: &str) -> Option<Self> {
        let (major, minor) = text.split_once('.')?;
        let major = u8::from_str_radix(major, 10).ok()?;
        let minor = u8::from_str_radix(minor, 10).ok()?;
        Some(Version(major, minor))
    }
}

/// Where a function is available to be called.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Availability {
    /// The function is required but missing.
    Missing,
    /// The function may be linked directly at build time.
    Link,
    /// The function must be loaded by pointer at runtime.
    Runtime,
}

/// A set of features included in an API.
struct FeatureSet<'a> {
    enums: HashSet<&'a str>,
    commands: HashMap<&'a str, Availability>,
}

impl<'a> FeatureSet<'a> {
    fn build(node: Node<'a, 'a>, entry_points: &[&'a str]) -> Result<Self, GenerateErrorRange<'a>> {
        assert_eq!(node.tag_name().name(), "registry");
        let mut set = FeatureSet {
            enums: HashSet::new(),
            commands: HashMap::with_capacity(entry_points.len()),
        };
        for &name in entry_points.iter() {
            set.commands.insert(name, Availability::Missing);
        }
        for child in node.children() {
            if child.is_element() && child.tag_name().name() == "feature" {
                set.parse_feature(child)?;
            }
        }
        for &name in entry_points.iter() {
            if !set.commands.contains_key(name) {
                return Err((GenerateError::MissingCommand(name.to_string()), None));
            }
        }
        Ok(set)
    }

    fn parse_feature(&mut self, node: Node<'a, 'a>) -> Result<(), GenerateErrorRange<'a>> {
        assert_eq!(node.tag_name().name(), "feature");
        if require_attribute(node, "api")? != "gl" {
            return Ok(());
        }
        let version = require_attribute(node, "number")?;
        let version = match Version::parse(version) {
            None => {
                return Err((
                    GenerateError::InvalidVersion(version.to_string()),
                    Some((node.tag_name().name(), node.range())),
                ));
            }
            Some(version) => version,
        };
        let availability = if version <= LINKABLE_VERSION {
            Availability::Link
        } else if version <= MAX_VERSION {
            Availability::Runtime
        } else {
            return Ok(());
        };
        for child in node.children() {
            if child.is_element() {
                match child.tag_name().name() {
                    "require" => self.parse_require(child, availability)?,
                    "remove" => self.parse_remove(child)?,
                    _ => return Err(unexpected_tag(node, child)),
                }
            }
        }
        Ok(())
    }

    fn parse_require(
        &mut self,
        node: Node<'a, 'a>,
        availability: Availability,
    ) -> Result<(), GenerateErrorRange<'a>> {
        assert_eq!(node.tag_name().name(), "require");
        for child in node.children() {
            if child.is_element() {
                match child.tag_name().name() {
                    "command" => {
                        let name = require_attribute(child, "name")?;
                        match self.commands.get_mut(name) {
                            None => (),
                            Some(value) => *value = availability,
                        }
                    }
                    "enum" => {
                        let name = require_attribute(child, "name")?;
                        self.enums.insert(name);
                    }
                    "type" => (),
                    _ => return Err(unexpected_tag(node, child)),
                }
            }
        }
        Ok(())
    }

    fn parse_remove(&mut self, node: Node<'a, 'a>) -> Result<(), GenerateErrorRange<'a>> {
        assert_eq!(node.tag_name().name(), "remove");
        let profile = require_attribute(node, "profile")?;
        if profile != "core" {
            return Err((
                GenerateError::InvalidRemoveProfile,
                Some((node.tag_name().name(), node.range())),
            ));
        }
        for child in node.children() {
            if child.is_element() {
                match child.tag_name().name() {
                    "command" => {
                        let name = require_attribute(child, "name")?;
                        match self.commands.get_mut(name) {
                            None => (),
                            Some(value) => *value = Availability::Missing,
                        }
                    }
                    "enum" => {
                        let name = require_attribute(child, "name")?;
                        self.enums.remove(name);
                    }
                    "type" => (),
                    _ => return Err(unexpected_tag(node, child)),
                }
            }
        }
        Ok(())
    }
}

// ============================================================================

fn element_children<'a>(node: Node<'a, 'a>) -> impl Iterator<Item = Node<'a, 'a>> {
    node.children().filter(|c| c.is_element())
}

fn element_children_tag<'a>(
    node: Node<'a, 'a>,
    name: &'static str,
) -> impl Iterator<Item = Node<'a, 'a>> {
    node.children()
        .filter(move |c| c.is_element() && c.tag_name().name() == name)
}

/// Emit enum value definitions.
fn emit_enums<'a>(
    enums: &HashSet<&str>,
    node: Node<'a, 'a>,
) -> Result<String, GenerateErrorRange<'a>> {
    let mut out = String::new();
    let mut emitted = HashSet::with_capacity(enums.len());
    for child in element_children_tag(node, "enums") {
        let ty = match child.attribute("type") {
            None => "GLenum",
            Some(s) => match s {
                "bitmask" => "GLbitfield",
                _ => panic!("type {:?}", s),
            },
        };
        for item in element_children(child) {
            match item.tag_name().name() {
                "enum" => {
                    if let Some(api) = item.attribute("api") {
                        if api != "gl" {
                            continue;
                        }
                    }
                    let name = require_attribute(item, "name")?;
                    if !enums.contains(name) {
                        continue;
                    }
                    if emitted.contains(name) {
                        return Err((
                            GenerateError::DuplicateEnum(name.to_string()),
                            Some((item.tag_name().name(), item.range())),
                        ));
                    }
                    emitted.insert(name);
                    let value = require_attribute(item, "value")?;
                    writeln!(out, "constexpr {} {} = {};", ty, name, value).unwrap();
                    if let Some(alias) = item.attribute("alias") {
                        writeln!(out, "constexpr {} {} = {};", ty, alias, name).unwrap();
                    }
                }
                "unused" => (),
                name => return Err(unexpected_tag(child, item)),
            }
        }
    }
    Ok(out)
}

/// Add a command to the list of commands, if it is requested (in the entry
/// map).
fn add_command<'a, 'input>(
    node: Node<'a, 'input>,
    entry_map: &mut HashMap<&str, bool>,
    entry_list: &mut Vec<(String, Node<'a, 'input>)>,
) -> Result<(), GenerateErrorRange<'input>> {
    assert_eq!(node.tag_name().name(), "command");
    let Some(proto) = node
        .children()
        .find(|node| node.is_element() && node.tag_name().name() == "proto")
    else {
        return Err((
            GenerateError::MissingCommandProto,
            Some((node.tag_name().name(), node.range())),
        ));
    };
    let Some(name_node) = proto
        .children()
        .find(|node| node.is_element() && node.tag_name().name() == "name")
    else {
        return Err((
            GenerateError::MissingCommandName,
            Some((proto.tag_name().name(), proto.range())),
        ));
    };
    let name = parse_text_contents(name_node)?;
    let Some(value) = entry_map.get_mut(name.as_str()) else {
        // Entry point is not requested.
        return Ok(());
    };
    if *value {
        return Err((
            GenerateError::DuplicateCommand(name),
            Some((name_node.tag_name().name(), name_node.range())),
        ));
    }
    entry_list.push((name, node));
    *value = true;
    Ok(())
}

fn add_commands<'input>(
    node: Node<'_, 'input>,
    entry_points: &[&str],
) -> Result<(), GenerateErrorRange<'input>> {
    // Create a map of all commands.
    let mut entry_map: HashMap<&str, bool> = HashMap::with_capacity(entry_points.len());
    for &name in entry_points.iter() {
        entry_map.insert(name, false);
    }
    let mut entry_list = Vec::with_capacity(entry_points.len());

    // Get the XML nodes for the commands.
    for node in node.children() {
        if node.is_element() && node.tag_name().name() == "commands" {
            for node in node.children() {
                if node.is_element() && node.tag_name().name() == "command" {
                    add_command(node, &mut entry_map, &mut entry_list)?;
                }
            }
        }
    }

    // Check if any are missing.
    for &name in entry_points.iter() {
        match entry_map.get(name) {
            Some(false) => return Err((GenerateError::MissingCommand(name.to_string()), None)),
            _ => (),
        }
    }
    todo!()
}

pub fn generate_doc<'a>(
    entry_points: &[&'a str],
    root: Node<'a, 'a>,
) -> Result<(), GenerateErrorRange<'a>> {
    let features = FeatureSet::build(root, entry_points)?;
    for &name in features.enums.iter() {
        eprintln!("Enum: {}", name);
    }
    for (&name, &linkage) in features.commands.iter() {
        eprintln!("Command: {} {:?}", name, linkage);
    }
    let enums = emit_enums(&features.enums, root)?;
    eprint!("{}", enums);
    Ok(())
}

pub fn generate(entry_points: &[&str]) -> Result<(), Error> {
    let spec_data = khronos_api::GL_XML;
    let spec_data = str::from_utf8(spec_data).expect("XML registry is not UTF-8.");
    let doc = match Document::parse(spec_data) {
        Ok(doc) => doc,
        Err(err) => return Err(Error::XML(err)),
    };
    let root = doc.root_element();
    match generate_doc(entry_points, root) {
        Ok(()) => Ok(()),
        Err((error, pos)) => Err(Error::Generate(GenerateErrorPos {
            error,
            pos: pos.map(|(tag, text_range)| (tag.to_string(), doc.text_pos_at(text_range.start))),
        })),
    }
}
