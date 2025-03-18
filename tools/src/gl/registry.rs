use core::error;
use core::str;
use khronos_api;
use roxmltree::NodeType;
use roxmltree::{self, Document, Node};
use std::fmt;

const APIENTRY: &str = "GLAPIENTRY";

/// A type definition in the OpenGL API.
#[derive(Debug, Clone)]
struct Type {
    name: Option<String>,
    definition: String,
}

#[derive(Debug, Clone)]
pub enum ParseError {
    XML(roxmltree::Error),
    UnexpectedTag(String, &'static str),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::XML(e) => e.fmt(f),
            ParseError::UnexpectedTag(tag, expected) => {
                write!(f, "unexpected tag: <{}> (expected <{}>)", tag, expected)
            }
        }
    }
}

impl error::Error for ParseError {}

impl From<roxmltree::Error> for ParseError {
    fn from(value: roxmltree::Error) -> Self {
        ParseError::XML(value)
    }
}

fn expect_tag(node: Node, tag: &'static str) -> Result<(), ParseError> {
    let name = node.tag_name().name();
    if name == tag {
        Ok(())
    } else {
        Err(ParseError::UnexpectedTag(name.to_string(), tag))
    }
}

/// Parse an element which only contains text. Return the text.
fn parse_text_contents(node: Node) -> Result<String, ParseError> {
    let mut result = String::new();
    for child in node.children() {
        match child.node_type() {
            NodeType::Text => {
                if let Some(text) = child.text() {
                    result.push_str(text);
                }
            }
            NodeType::Element => eprintln!(
                "Unknown tag in <{}>: <{}>",
                node.tag_name().name(),
                child.tag_name().name()
            ),
            _ => (),
        }
    }
    Ok(result)
}

/// Parse a <type> tag.
fn parse_type(node: Node) -> Result<Type, ParseError> {
    expect_tag(node, "type")?;
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
fn parse_types(node: Node) -> Result<(), ParseError> {
    expect_tag(node, "types")?;
    for child in node.children() {
        if child.node_type() == NodeType::Element {
            match child.tag_name().name() {
                "type" => {
                    let ty = parse_type(child)?;
                    eprintln!("Type: {:?}", ty);
                }
                other => eprintln!("Unknown tag in <types>: <{}>", other),
            }
        }
    }
    Ok(())
}

/// Parse a <registry> tag.
fn parse_registry(node: Node) -> Result<(), ParseError> {
    expect_tag(node, "registry")?;
    for child in node.children() {
        if child.node_type() == NodeType::Element {
            match child.tag_name().name() {
                "types" => {
                    parse_types(child)?;
                }
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
    Ok(())
}

pub fn run() -> Result<(), ParseError> {
    let data = khronos_api::GL_XML;
    let data = str::from_utf8(data).expect("XML registry is not UTF-8.");
    let doc = Document::parse(data)?;
    let node = doc.root_element();
    parse_registry(node)?;
    Ok(())
}
