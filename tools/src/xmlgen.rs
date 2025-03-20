use std::fmt::Write as _;

/// Delimiter for text not in an attribute.
const NO_DELIM: char = '\0';

/// Write quoted XML data to a string. If we are in an attribute, pass the
/// attribute delimeter in. Otherwise, use NO_DELIM.
fn quote(out: &mut String, text: &str, delim: char) {
    // 2.2 Characetrs - defines range as 0x09, 0x0a, 0x0d, then anything from 0x20 up.
    // AttValue: [^<&"] / [^<&']
    for c in text.chars() {
        match c {
            '\x09' | '\x0a' | '\x0d' => out.push(c),
            '\x00'..='\x1f' => write!(out, "&x{:x};", u32::from(c)).unwrap(),
            '<' => out.push_str("&lt;"),
            '&' => out.push_str("&amp;"),
            '"' => {
                if delim == '"' {
                    out.push_str("&quot;");
                } else {
                    out.push(c);
                }
            }
            '\'' => {
                if delim == '\'' {
                    out.push_str("&apos;");
                } else {
                    out.push('\'');
                }
            }
            _ => out.push(c),
        }
    }
}

/// Insert a line break and indent to the given depth.
fn new_line(out: &mut String, indent: usize) {
    out.push_str("\r\n");
    for _ in 0..indent {
        out.push(' ');
    }
}

/// Choose the preferred delimiter for an attribute value.
fn choose_delim(text: &str) -> char {
    let mut n: i32 = 0;
    for c in text.bytes() {
        match c {
            b'\'' => n += 1,
            b'"' => n -= 1,
            _ => (),
        }
    }
    if n >= 0 { '"' } else { '\'' }
}

pub struct XML {
    text: String,
}

impl XML {
    /// Create a new XML document.
    pub fn new() -> XML {
        XML {
            text: "<?xml version=\"1.0\" encoding=\"utf-8\"?>".to_string(),
        }
    }

    /// Create the root element in the document.
    pub fn root<'a>(&'a mut self, tag: &'a str) -> Tag<'a> {
        let s = &mut self.text;
        new_line(s, 0);
        s.push_str("<");
        s.push_str(tag);
        Tag {
            xml: self,
            tag,
            indent: 0,
        }
    }

    /// Finish constructing the XML document.
    pub fn finish(self) -> String {
        self.text
    }
}

/// An XML tag. Attributes and content can be added to the tag.
pub struct Tag<'a> {
    xml: &'a mut XML,
    tag: &'a str,
    indent: usize,
}

impl<'a> Tag<'a> {
    /// Add an attribute to the element.
    pub fn add_attr(&mut self, name: &str, value: &str) {
        let delim = choose_delim(value);
        let s = &mut self.xml.text;
        s.push(' ');
        s.push_str(name);
        s.push('=');
        s.push(delim);
        quote(s, value, delim);
        s.push(delim);
    }

    /// Add an attribute to the element, returning it.
    pub fn attr(mut self, name: &str, value: &str) -> Self {
        self.add_attr(name, value);
        self
    }

    /// Self-close the tag.
    pub fn close(self) {
        self.xml.text.push_str(" />");
    }

    /// Put text content in the tag and close it.
    pub fn text(self, text: &str) {
        let s = &mut self.xml.text;
        s.push('>');
        quote(s, text, NO_DELIM);
        s.push_str("</");
        s.push_str(self.tag);
        s.push_str(">");
    }

    pub fn open(self) -> Element<'a> {
        self.xml.text.push_str(">");
        Element {
            xml: self.xml,
            tag: self.tag,
            indent: self.indent,
        }
    }
}

/// An XML element to which other elements can be added.
pub struct Element<'a> {
    xml: &'a mut XML,
    tag: &'a str,
    indent: usize,
}

impl<'a> Element<'a> {
    /// Open a new tag inside this element.
    pub fn tag<'b>(&'b mut self, tag: &'b str) -> Tag<'b> {
        let s = &mut self.xml.text;
        let indent = self.indent + 2;

        new_line(s, indent);
        s.push('<');
        s.push_str(tag);
        Tag {
            xml: self.xml,
            tag,
            indent,
        }
    }

    /// Close this element.
    pub fn close(self) {
        let s = &mut self.xml.text;
        new_line(s, self.indent);
        s.push_str("</");
        s.push_str(self.tag);
        s.push_str(">");
    }
}
