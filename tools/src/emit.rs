use std::fmt::Write;
use std::fs;
use std::io;
use std::io::Write as _;
use std::path::{Path, PathBuf};

const COLUMNS: usize = 79;

/// Header to put in front of generated C or C++ source files.
pub const HEADER: &str = "// This file is automatically generated.
// clang-format off
";

/// C string writer.
pub struct StringWriter<'a> {
    out: &'a mut String,
    limit: usize,
}

impl<'a> StringWriter<'a> {
    /// Create a new writer, which appends a C string to the output.
    pub fn new(out: &'a mut String) -> Self {
        out.push('"');
        let limit = out.len() + (COLUMNS - 2);
        StringWriter { out, limit }
    }

    /// Write the end of a string (the final quote).
    pub fn finish(self) {
        self.out.push_str("\"\n")
    }

    /// Append a quoted C string to the given string. This may be split across
    /// multiple lines.
    pub fn write(&mut self, text: &[u8]) {
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

/// Write a file to disk.
pub fn write(path: &Path, contents: &[u8]) -> io::Result<()> {
    eprintln!("Writing file: {}", path.display());
    fs::write(path, contents)
}

pub fn write_or_stdout(path: Option<&Path>, contents: &[u8]) -> io::Result<()> {
    match path {
        None => io::stdout().lock().write_all(contents),
        Some(path) => write(path, contents),
    }
}

/// A collection of outputs to emit.
pub struct Outputs {
    files: Vec<(PathBuf, Vec<u8>)>,
}

impl Outputs {
    pub fn new() -> Self {
        Outputs { files: Vec::new() }
    }

    /// Add a file to the outputs.
    pub fn add_file(&mut self, path: impl Into<PathBuf>, data: impl Into<Vec<u8>>) {
        self.files.push((path.into(), data.into()));
    }

    /// Write outputs to the filesystem.
    pub fn write(self) -> io::Result<()> {
        for (path, data) in self.files {
            write(&path, &data)?;
        }
        Ok(())
    }
}
