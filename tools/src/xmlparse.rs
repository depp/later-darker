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
    UnexpectedText {
        text: String,
        parent: String,
        pos: TextPos,
    },
    UnexpectedPI {
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
            Error::UnexpectedText { text, parent, pos } => {
                write!(f, "unexpected text {:?} at {} in <{}>", text, parent, pos)
            }
            Error::UnexpectedPI { parent, pos } => write!(
                f,
                "unexpected processing instruction at {} in <{}>",
                pos, parent
            ),
        }
    }
}

impl error::Error for Error {}

/// Create an error for an unexpected tag.
pub fn unexpected_tag(node: Node) -> Error {
    if let Some(parent) = node.parent() {
        let parent = parent.tag_name().name();
        if !parent.is_empty() {
            return Error::UnexpectedTag {
                tag: node.tag_name().name().into(),
                parent: parent.into(),
                pos: node_pos(node),
            };
        }
    }
    Error::UnexpectedRoot {
        tag: node.tag_name().name().into(),
        pos: node_pos(node),
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

/// Check that the node has no attributes.
pub fn check_no_attributes(node: Node) -> Result<(), Error> {
    match node.attributes().next() {
        None => Ok(()),
        Some(attr) => Err(unexpected_attribute(node, attr)),
    }
}

/// Create an error for a missing required attribute.
pub fn missing_attribute(node: Node, name: &str) -> Error {
    Error::MissingAttribute {
        attribute: name.into(),
        parent: node.tag_name().name().into(),
        pos: node.document().text_pos_at(node.range().start),
    }
}

/// Get a required attribute from a node, or return an error if the attribute is
/// not present.
pub fn require_attribute<'a>(node: Node<'a, '_>, name: &str) -> Result<&'a str, Error> {
    match node.attribute(name) {
        None => Err(missing_attribute(node, name)),
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
                return Err(unexpected_tag(child));
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

/// Create an error for unexpected text content.
pub fn unexpected_text(node: Node, offset: usize) -> Error {
    let text = node.text().expect("Must be a text node");
    let mut chars = text.chars();
    chars.next();
    Error::UnexpectedText {
        text: text[..text.len() - chars.as_str().len()].into(),
        parent: node
            .parent()
            .expect("Must have a parent")
            .tag_name()
            .name()
            .into(),
        pos: node.document().text_pos_at(node.range().start + offset),
    }
}

/// Iterate over children of a node.
pub fn element_children_unchecked<'a>(node: Node<'a, 'a>) -> impl Iterator<Item = Node<'a, 'a>> {
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

/// Return the children of a node, which must be elements.
pub fn elements_children<'a, 'input>(node: Node<'a, 'input>) -> ElementsChecked<'a, 'input> {
    ElementsChecked(node.children())
}

/// Iterator over children of an element, which must themselves be elements.
#[derive(Clone)]
pub struct ElementsChecked<'a, 'input>(roxmltree::Children<'a, 'input>);

fn element_checked<'a, 'input>(node: Node<'a, 'input>) -> Option<Result<Node<'a, 'input>, Error>> {
    match node.node_type() {
        NodeType::Root => None,
        NodeType::Element => Some(Ok(node)),
        NodeType::PI => Some(Err(Error::UnexpectedPI {
            parent: node
                .parent()
                .expect("Must have parent")
                .tag_name()
                .name()
                .into(),
            pos: node.document().text_pos_at(node.range().start),
        })),
        NodeType::Comment => None,
        NodeType::Text => {
            if let Some(text) = node.text() {
                let non_space = text.trim_ascii_start();
                if !non_space.is_empty() {
                    return Some(Err(unexpected_text(node, text.len() - non_space.len())));
                }
            }
            None
        }
    }
}

impl<'a, 'input> Iterator for ElementsChecked<'a, 'input> {
    type Item = Result<Node<'a, 'input>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.0.next() {
                None => return None,
                Some(node) => {
                    let value = element_checked(node);
                    if value.is_some() {
                        return value;
                    }
                }
            }
        }
    }
}

impl<'a, 'input> DoubleEndedIterator for ElementsChecked<'a, 'input> {
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            match self.0.next_back() {
                None => return None,
                Some(node) => {
                    let value = element_checked(node);
                    if value.is_some() {
                        return value;
                    }
                }
            }
        }
    }
}
