use core::error;
use core::str;
use khronos_api;
use roxmltree::NodeType;
use roxmltree::{self, Document, Node};
use std::fmt;

#[derive(Debug, Clone)]
pub enum ParseError {
    XML(roxmltree::Error),
    UnexpectedTag(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::XML(e) => e.fmt(f),
            ParseError::UnexpectedTag(tag) => write!(f, "unexpected tag: <{}>", tag),
        }
    }
}

impl error::Error for ParseError {}

impl From<roxmltree::Error> for ParseError {
    fn from(value: roxmltree::Error) -> Self {
        ParseError::XML(value)
    }
}

fn expect_tag(node: Node, tag: &str) -> Result<(), ParseError> {
    let name = node.tag_name().name();
    if name == tag {
        Ok(())
    } else {
        Err(ParseError::UnexpectedTag(name.to_string()))
    }
}

fn parse_registry(node: Node) -> Result<(), ParseError> {
    expect_tag(node, "registry")?;
    for child in node.children() {
        match child.node_type() {
            NodeType::Element => eprintln!(" <{}>", child.tag_name().name()),
            _ => (),
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
