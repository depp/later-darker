use core::error;
use core::str;
use khronos_api;
use roxmltree::{self, Document, Node, NodeType};
use std::collections::HashMap;
use std::fmt;
use std::ops::Range;

const APIENTRY: &str = "GLAPIENTRY";

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
        }
    }
}

impl error::Error for GenerateError {}

type GenerateErrorRange<'input> = (GenerateError, Option<(&'input str, Range<usize>)>);

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

pub fn generate(entry_points: &[&str]) -> Result<(), Error> {
    let spec_data = khronos_api::GL_XML;
    let spec_data = str::from_utf8(spec_data).expect("XML registry is not UTF-8.");
    let doc = match Document::parse(spec_data) {
        Ok(doc) => doc,
        Err(err) => return Err(Error::XML(err)),
    };
    let root = doc.root_element();
    match add_commands(root, entry_points) {
        Ok(()) => (),
        Err((error, pos)) => {
            return Err(Error::Generate(GenerateErrorPos {
                error,
                pos: pos
                    .map(|(tag, text_range)| (tag.to_string(), doc.text_pos_at(text_range.start))),
            }));
        }
    };
    Ok(())
}
