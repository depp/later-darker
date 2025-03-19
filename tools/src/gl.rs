use khronos_api;
use roxmltree::{self, Document, Node, NodeType};
use std::collections::{HashMap, HashSet};
use std::error;
use std::fmt::{self, Write as _};
use std::ops::Range;
use std::rc::Rc;
use std::str;

const LINKABLE_VERSION: Version = Version(1, 1);
const MAX_VERSION: Version = Version(3, 3);

#[derive(Debug, Clone)]
pub enum ErrorKind {
    UnexpectedTag(String),
    MissingCommandProto,
    MissingCommandName,
    MissingCommand(String),
    MissingAttribute(&'static str),
    InvalidVersion(String),
    InvalidRemoveProfile,
    DuplicateEnum(String),
    DuplicateFunction(String),
    InvalidPrototype,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ErrorKind::*;
        match self {
            UnexpectedTag(tag) => write!(f, "unexpected tag: <{}>", tag),
            MissingCommandProto => f.write_str("missing command <proto>"),
            MissingCommandName => f.write_str("missing command <name>"),
            MissingCommand(name) => {
                write!(f, "could not find command definition: {:?}", name)
            }
            MissingAttribute(name) => {
                write!(f, "missing required attribute: {}", name)
            }
            InvalidVersion(text) => write!(f, "invalid version number: {:?}", text),
            InvalidRemoveProfile => write!(f, "invalid profile for remove"),
            DuplicateEnum(name) => write!(f, "duplicate enum: {:?}", name),
            DuplicateFunction(name) => write!(f, "dupliacte function: {:?}", name),
            InvalidPrototype => write!(f, "invalid prototype"),
        }
    }
}

impl ErrorKind {
    /// Return this error with the location for a node included.
    fn at_node<'a>(self, node: Node<'_, 'a>) -> RError<'a> {
        RError {
            kind: self,
            pos: Some((node.tag_name().name(), node.range())),
        }
    }
}

impl error::Error for ErrorKind {}

/// A code generation error. This is an internal version, which is converted ot the GError below.
struct RError<'a> {
    kind: ErrorKind,
    pos: Option<(&'a str, Range<usize>)>,
}

impl<'a> From<ErrorKind> for RError<'a> {
    fn from(value: ErrorKind) -> Self {
        RError {
            kind: value,
            pos: None,
        }
    }
}

fn unexpected_tag<'a, 'input>(parent: Node<'a, 'input>, child: Node<'a, 'input>) -> RError<'input> {
    RError {
        kind: ErrorKind::UnexpectedTag(child.tag_name().name().to_string()),
        pos: Some((parent.tag_name().name(), child.range())),
    }
}

/// A code generation error.
#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
    pub pos: Option<(String, roxmltree::TextPos)>,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.pos {
            None => self.kind.fmt(f),
            Some((tag, pos)) => write!(f, "line {}: in <{}>: {}", pos.row, tag, self.kind),
        }
    }
}

#[derive(Debug)]
pub enum GenerateError {
    XML(roxmltree::Error),
    Generate(Error),
}

impl fmt::Display for GenerateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GenerateError::XML(e) => write!(f, "could not parse OpenGL spec: {}", e),
            GenerateError::Generate(e) => write!(f, "could not generate OpenGL API: {}", e),
        }
    }
}

impl error::Error for GenerateError {}

fn require_attribute<'a, 'input>(
    node: Node<'a, 'input>,
    name: &'static str,
) -> Result<&'a str, RError<'input>> {
    match node.attribute(name) {
        None => Err(ErrorKind::MissingAttribute(name).at_node(node)),
        Some(text) => Ok(text),
    }
}

/// Append the text contents of a node to the given string. The node must contain only text.
fn append_text_contents<'a>(out: &mut String, node: Node<'_, 'a>) -> Result<(), RError<'a>> {
    for child in node.children() {
        match child.node_type() {
            NodeType::Text => {
                if let Some(text) = child.text() {
                    out.push_str(text);
                }
            }
            NodeType::Element => {
                return Err(unexpected_tag(node, child));
            }
            _ => (),
        }
    }
    Ok(())
}

/// Parse an element which only contains text. Return the text.
fn parse_text_contents<'input>(node: Node<'_, 'input>) -> Result<String, RError<'input>> {
    let mut out = String::new();
    append_text_contents(&mut out, node)?;
    Ok(out)
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
    fn build(node: Node<'a, 'a>, entry_points: &[&'a str]) -> Result<Self, RError<'a>> {
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
                return Err(ErrorKind::MissingCommand(name.to_string()).into());
            }
        }
        Ok(set)
    }

    fn parse_feature(&mut self, node: Node<'a, 'a>) -> Result<(), RError<'a>> {
        assert_eq!(node.tag_name().name(), "feature");
        if require_attribute(node, "api")? != "gl" {
            return Ok(());
        }
        let version = require_attribute(node, "number")?;
        let version = match Version::parse(version) {
            None => {
                return Err(ErrorKind::InvalidVersion(version.to_string()).at_node(node));
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
    ) -> Result<(), RError<'a>> {
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

    fn parse_remove(&mut self, node: Node<'a, 'a>) -> Result<(), RError<'a>> {
        assert_eq!(node.tag_name().name(), "remove");
        let profile = require_attribute(node, "profile")?;
        if profile != "core" {
            return Err(ErrorKind::InvalidRemoveProfile.at_node(node));
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
fn emit_enums<'a>(enums: &HashSet<&str>, node: Node<'a, 'a>) -> Result<String, RError<'a>> {
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
                        return Err(ErrorKind::DuplicateEnum(name.to_string()).at_node(item));
                    }
                    emitted.insert(name);
                    let value = require_attribute(item, "value")?;
                    writeln!(out, "constexpr {} {} = {};", ty, name, value).unwrap();
                    if let Some(alias) = item.attribute("alias") {
                        writeln!(out, "constexpr {} {} = {};", ty, alias, name).unwrap();
                    }
                }
                "unused" => (),
                _ => return Err(unexpected_tag(child, item)),
            }
        }
    }
    Ok(out)
}

/// Get the name and prototype for a command.
fn command_info<'a>(node: Node<'a, 'a>) -> Result<(String, Node<'a, 'a>), RError<'a>> {
    assert_eq!(node.tag_name().name(), "command");
    let Some(proto) = element_children_tag(node, "proto").next() else {
        return Err(ErrorKind::MissingCommandProto.at_node(node));
    };
    let Some(name) = element_children_tag(proto, "name").next() else {
        return Err(ErrorKind::MissingCommandName.at_node(proto));
    };
    Ok((parse_text_contents(name)?, proto))
}

/// Emit the return type of a function, given the <proto> tag.
fn emit_return_type<'a>(node: Node<'a, 'a>) -> Result<String, RError<'a>> {
    let mut out = String::new();
    let mut has_name = false;
    for child in node.children() {
        match child.node_type() {
            NodeType::Element => match child.tag_name().name() {
                "name" => has_name = true,
                "ptype" => {
                    if has_name {
                        return Err(ErrorKind::InvalidPrototype.at_node(node));
                    }
                    let ty = parse_text_contents(child)?;
                    out.push_str(&ty);
                }
                _ => return Err(unexpected_tag(node, child)),
            },
            NodeType::Text => {
                if let Some(text) = child.text() {
                    if !has_name {
                        out.push_str(text);
                    } else if text.chars().any(|c| !c.is_ascii_whitespace()) {
                        return Err(ErrorKind::InvalidPrototype.at_node(node));
                    }
                }
            }
            _ => (),
        }
    }
    let len = out.trim_ascii_end().len();
    if len == 0 {
        return Err(ErrorKind::InvalidPrototype.at_node(node));
    }
    out.truncate(len);
    Ok(out)
}

/// Emit the parameter declarations and parameter names, given the <command>
/// tag.
fn emit_parameters<'a>(node: Node<'a, 'a>) -> Result<(String, String), RError<'a>> {
    let mut declarations = String::new();
    let mut names = String::new();
    let mut has_parameter = false;
    for child in element_children_tag(node, "param") {
        if has_parameter {
            declarations.push_str(", ");
            names.push_str(", ");
        }
        has_parameter = true;
        let mut has_name = false;
        for item in child.children() {
            match item.node_type() {
                NodeType::Element => match item.tag_name().name() {
                    "ptype" => append_text_contents(&mut declarations, item)?,
                    "name" => {
                        if has_name {
                            return Err(ErrorKind::InvalidPrototype.at_node(item));
                        }
                        has_name = true;
                        let pos = declarations.len();
                        append_text_contents(&mut declarations, item)?;
                        names.push_str(&declarations[pos..]);
                    }
                    _ => return Err(unexpected_tag(child, item)),
                },
                NodeType::Text => {
                    if let Some(text) = item.text() {
                        declarations.push_str(text);
                    }
                }
                _ => (),
            }
        }
        if !has_name {
            return Err(ErrorKind::InvalidPrototype.at_node(child));
        }
    }
    Ok((declarations, names))
}

/// Emit OpenGL function interfaces.
fn emit_functions<'a>(
    commands: &HashMap<&'a str, Availability>,
    node: Node<'a, 'a>,
) -> Result<String, RError<'a>> {
    let mut out = String::new();
    let mut lookups = Vec::with_capacity(commands.len());
    let mut emitted: HashSet<Rc<str>> = HashSet::with_capacity(commands.len());
    for child in element_children_tag(node, "commands") {
        for item in element_children(child) {
            if item.tag_name().name() != "command" {
                return Err(unexpected_tag(child, item));
            }
            let (name, proto) = command_info(item)?;
            let Some(&availability) = commands.get(name.as_str()) else {
                continue;
            };
            if emitted.contains(name.as_str()) {
                return Err(ErrorKind::DuplicateFunction(name).at_node(item));
            }
            let name: Rc<str> = name.into();
            let return_type = emit_return_type(proto)?;
            let (declarations, names) = emit_parameters(item)?;
            eprintln!(
                "ret={:?} decl={:?} name={:?}",
                return_type, declarations, names
            );
            match availability {
                Availability::Missing => (), // FIXME: error!
                Availability::Link => {
                    write!(
                        out,
                        "{} GLIMPORT {}({});\n",
                        return_type, name, declarations
                    )
                    .unwrap();
                }
                Availability::Runtime => {
                    let index = lookups.len();
                    lookups.push(name.clone());
                    write!(
                        out,
                        "inline {} {}({}) {{\n\
                        using Proc = {} (GLAPI *)({});\n",
                        return_type, name, declarations, return_type, declarations
                    )
                    .unwrap();
                    if return_type != "void" {
                        out.push_str("return ");
                    }
                    write!(
                        out,
                        "static_cast<Proc>(GLFunctionPointers[{}])({});\n}}\n",
                        index, names
                    )
                    .unwrap();
                }
            }
            emitted.insert(name);
        }
    }
    Ok(out)
}

fn generate_doc<'a>(entry_points: &[&'a str], root: Node<'a, 'a>) -> Result<(), RError<'a>> {
    let features = FeatureSet::build(root, entry_points)?;
    let enums = emit_enums(&features.enums, root)?;
    eprint!("{}", enums);
    let functions = emit_functions(&features.commands, root)?;
    eprint!("{}", functions);
    Ok(())
}

pub fn generate(entry_points: &[&str]) -> Result<(), GenerateError> {
    let spec_data = khronos_api::GL_XML;
    let spec_data = str::from_utf8(spec_data).expect("XML registry is not UTF-8.");
    let doc = match Document::parse(spec_data) {
        Ok(doc) => doc,
        Err(err) => return Err(GenerateError::XML(err)),
    };
    let root = doc.root_element();
    match generate_doc(entry_points, root) {
        Ok(()) => Ok(()),
        Err(RError { kind, pos }) => Err(GenerateError::Generate(Error {
            kind,
            pos: pos.map(|(tag, text_range)| (tag.to_string(), doc.text_pos_at(text_range.start))),
        })),
    }
}
