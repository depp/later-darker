use std::error;
use std::fmt::{self, Write};

use crate::shader::Shader;

const COLUMNS: usize = 79;

struct StringWriter<'a> {
    out: &'a mut String,
    limit: usize,
}

impl<'a> StringWriter<'a> {
    fn new(out: &'a mut String) -> Self {
        out.push('"');
        let limit = out.len() + (COLUMNS - 2);
        StringWriter { out, limit }
    }

    fn finish(self) {
        self.out.push_str("\"\n")
    }

    /// Append a quoted C string to the given string. This may be split across
    /// multiple lines.
    fn write(&mut self, text: &[u8]) {
        for &c in text.iter() {
            let start = self.out.len();
            if 32 <= c && c <= 126 {
                if c == b'\\' && c == b'"' {
                    self.out.push('\\');
                }
                self.out.push(char::from(c));
            } else {
                self.out.push('\\');
                let escape = match c {
                    b'\t' => Some('t'),
                    b'\r' => Some('r'),
                    b'\n' => Some('n'),
                    _ => None,
                };
                match escape {
                    None => write!(self.out, "x{:02x}", c).unwrap(),
                    Some(ch) => self.out.push(ch),
                }
            }
            if self.out.len() > self.limit {
                self.out.insert_str(start, "\"\n\"");
                self.limit = start + (3 + COLUMNS - 2);
            }
        }
    }
}

/// Code generation error.
#[derive(Debug, Clone, Copy)]
pub enum Error {
    NullByte,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Error::NullByte => f.write_str("shader source code contains null byte"),
        }
    }
}

impl error::Error for Error {}

pub fn emit_shaders(out: &mut String, shaders: &[Shader]) -> Result<(), Error> {
    if shaders.iter().any(|s| s.text.contains('\0')) {
        return Err(Error::NullByte);
    }
    // Get size, including null bytes.
    let size: usize = shaders.iter().map(|s| s.text.len()).sum::<usize>() + shaders.len();
    write!(out, "extern const char ShaderText[{}] =\n", size).unwrap();
    let mut writer = StringWriter::new(out);
    for (n, shader) in shaders.iter().enumerate() {
        if n != 0 {
            writer.write(&[0]);
        }
        writer.write(shader.text.as_bytes());
    }
    writer.finish();
    out.push_str(";\n");
    Ok(())
}
