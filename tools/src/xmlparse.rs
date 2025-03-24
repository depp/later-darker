use roxmltree::{Attribute, Node, NodeType, TextPos};
use std::error;
use std::fmt;

/// Get the text position for a node.
pub fn node_pos(node: Node) -> TextPos {
    node.document().text_pos_at(node.range().start)
}

/// Get the text position for an attribute.
pub fn attr_pos(node: Node, attr: Attribute) -> TextPos {
    node.document().text_pos_at(attr.range().start)
}

/// An error from parsing an XML document.
#[derive(Debug)]
pub enum Error {
    UnexpectedRoot {
        tag: String,
        pos: TextPos,
    },
    UnexpectedTag {
        tag: String,
        parent: String,
        pos: TextPos,
    },
    MissingAttribute {
        attribute: String,
        parent: String,
        pos: TextPos,
    },
    UnexpectedAttribute {
        attribute: String,
        parent: String,
        pos: TextPos,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::UnexpectedRoot { tag, pos } => {
                write!(f, "unexpected root tag <{}> at {}", tag, pos)
            }
            Error::UnexpectedTag { tag, parent, pos } => {
                write!(f, "unexpected tag <{}> at {} in <{}>", tag, pos, parent)
            }
            Error::MissingAttribute {
                attribute,
                parent,
                pos,
            } => write!(
                f,
                "missing required attribute '{}' in <{}> at {}",
                attribute, parent, pos
            ),
            Error::UnexpectedAttribute {
                attribute,
                parent,
                pos,
            } => write!(
                f,
                "unexpected attribute '{}' in <{}> at {}",
                attribute, parent, pos
            ),
        }
    }
}

impl error::Error for Error {}

/// Create an error for an unexpected tag.
pub fn unexpected_tag(node: Node, parent: Node) -> Error {
    Error::UnexpectedTag {
        tag: node.tag_name().name().into(),
        parent: parent.tag_name().name().into(),
        pos: node.document().text_pos_at(node.range().start),
    }
}

/// Create an error for an unexpected root tag.
pub fn unexpected_root(node: Node) -> Error {
    Error::UnexpectedRoot {
        tag: node.tag_name().name().into(),
        pos: node.document().text_pos_at(node.range().start),
    }
}

/// Create an error for an unexpected or unknown attribute.
pub fn unexpected_attribute(node: Node, attr: Attribute) -> Error {
    Error::UnexpectedAttribute {
        attribute: attr.name().into(),
        parent: node.tag_name().name().into(),
        pos: node.document().text_pos_at(node.range().start),
    }
}

/// Get a required attribute from a node, or return an error if the attribute is
/// not present.
pub fn require_attribute<'a>(node: Node<'a, '_>, name: &str) -> Result<&'a str, Error> {
    match node.attribute(name) {
        None => Err(Error::MissingAttribute {
            attribute: name.into(),
            parent: node.tag_name().name().into(),
            pos: node.document().text_pos_at(node.range().start),
        }),
        Some(value) => Ok(value),
    }
}

/// Append the text contents of a node to the given string. The node must contain only text.
pub fn append_text_contents(out: &mut String, node: Node) -> Result<(), Error> {
    for child in node.children() {
        match child.node_type() {
            NodeType::Text => {
                if let Some(text) = child.text() {
                    out.push_str(text);
                }
            }
            NodeType::Element => {
                return Err(unexpected_tag(child, node));
            }
            _ => (),
        }
    }
    Ok(())
}

/// Parse an element which only contains text. Return the text.
pub fn parse_text_contents(node: Node) -> Result<String, Error> {
    let mut out = String::new();
    append_text_contents(&mut out, node)?;
    Ok(out)
}

/// Iterate over children of a node.
pub fn element_children<'a>(node: Node<'a, 'a>) -> impl Iterator<Item = Node<'a, 'a>> {
    node.children().filter(|c| c.is_element())
}

/// Iterate over children of a node with the given tag.
pub fn element_children_tag<'a>(
    node: Node<'a, 'a>,
    name: &'static str,
) -> impl Iterator<Item = Node<'a, 'a>> {
    node.children()
        .filter(move |c| c.is_element() && c.tag_name().name() == name)
}
