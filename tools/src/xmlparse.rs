use arcstr::ArcStr;
use roxmltree::NodeType;
use roxmltree::{Node, TextPos};
use std::error;
use std::fmt;

/// Get the text position for a node.
pub fn node_pos(node: Node) -> TextPos {
    node.document().text_pos_at(node.range().start)
}

#[derive(Debug, Clone)]
pub struct TagPos {
    pub tag: ArcStr,
    pub pos: TextPos,
}

impl From<Node<'_, '_>> for TagPos {
    fn from(value: Node<'_, '_>) -> Self {
        TagPos {
            tag: ArcStr::from(value.tag_name().name()),
            pos: node_pos(value),
        }
    }
}

/// An error from parsing an XML document.
#[derive(Debug)]
pub enum Error<T> {
    XML(roxmltree::Error),
    UnexpectedRoot(TagPos),
    UnexpectedTag(TagPos, ArcStr),
    MissingAttribute(TagPos, ArcStr),
    Other(T),
}

impl<T> From<roxmltree::Error> for Error<T> {
    fn from(value: roxmltree::Error) -> Self {
        Self::XML(value)
    }
}

impl<T> fmt::Display for Error<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::XML(e) => e.fmt(f),
            Error::UnexpectedRoot(tag) => {
                write!(f, "unexpected root tag <{}> at {}", tag.tag, tag.pos)
            }
            Error::UnexpectedTag(tag, parent) => write!(
                f,
                "unexpected tag <{}> at {} in <{}>",
                tag.tag, tag.pos, parent
            ),

            Error::MissingAttribute(tag, attribute) => write!(
                f,
                "missing required attribute '{}' in <{}> at {}",
                attribute, tag.tag, tag.pos
            ),
            Error::Other(e) => e.fmt(f),
        }
    }
}

impl<T> error::Error for Error<T> where T: error::Error {}

/// Create an error for an unexpected tag.
pub fn unexpected_tag<T>(node: Node, parent: Node) -> Error<T> {
    Error::UnexpectedTag(node.into(), ArcStr::from(parent.tag_name().name()))
}

/// Create an error for an unexpected root tag.
pub fn unexpected_root<T>(node: Node) -> Error<T> {
    Error::UnexpectedRoot(node.into())
}

/// Get a required attribute from a node, or return an error if the attribute is
/// not present.
pub fn require_attribute<'a, T>(node: Node<'a, '_>, name: &str) -> Result<&'a str, Error<T>> {
    match node.attribute(name) {
        None => Err(Error::MissingAttribute(node.into(), ArcStr::from(name))),
        Some(value) => Ok(value),
    }
}

/// Append the text contents of a node to the given string. The node must contain only text.
pub fn append_text_contents<T>(out: &mut String, node: Node) -> Result<(), Error<T>> {
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
pub fn parse_text_contents<T>(node: Node) -> Result<String, Error<T>> {
    let mut out = String::new();
    append_text_contents(&mut out, node)?;
    Ok(out)
}
