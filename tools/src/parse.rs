use crate::spec::{Program, ShaderType};
use std::path::Path;
use std::{error, fmt, fs, io};

/// Kinds of parse errors.
#[derive(Debug)]
pub enum ErrorKind {
    UnknownField(String),
    UnknownExtension(String),
    NoShader(ShaderType),
    ExtraShader(ShaderType),
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorKind::UnknownField(text) => write!(f, "unknown field: {:?}", text),
            ErrorKind::UnknownExtension(ext) => write!(f, "unknown file extension: {:?}", ext),
            ErrorKind::NoShader(shader_type) => write!(f, "missing shader type: {:?}", shader_type),
            ErrorKind::ExtraShader(shader_type) => {
                write!(f, "multiple shaders with same type: {:?}", shader_type)
            }
        }
    }
}

/// Error parsing a spec.
#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    lineno: u32,
}

/// Parse a single line of program specs.
fn parse_line(line: &str) -> Result<Option<Program>, ErrorKind> {
    let line = match line.split_once('#') {
        None => line,
        Some((left, _)) => left,
    };
    let mut fields = line.split_ascii_whitespace();
    let name = match fields.next() {
        None => return Ok(None),
        Some(name) => name.to_string(),
    };
    let mut vertex: Option<String> = None;
    let mut fragment: Option<String> = None;
    for field in fields {
        if let Some((_, ext)) = field.rsplit_once('.') {
            let shader_type = match ShaderType::from_extension(ext) {
                None => return Err(ErrorKind::UnknownExtension(ext.to_string())),
                Some(shader_type) => shader_type,
            };
            let value = match shader_type {
                ShaderType::Vertex => &mut vertex,
                ShaderType::Fragment => &mut fragment,
            };
            if value.is_some() {
                return Err(ErrorKind::ExtraShader(shader_type));
            }
            *value = Some(field.to_string());
            continue;
        }
        return Err(ErrorKind::UnknownField(field.to_string()));
    }
    let vertex = vertex.ok_or(ErrorKind::NoShader(ShaderType::Vertex))?;
    let fragment = fragment.ok_or(ErrorKind::NoShader(ShaderType::Fragment))?;
    Ok(Some(Program {
        name,
        vertex,
        fragment,
    }))
}

/// Parse program specs from memory.
fn parse_spec(text: &str) -> Result<Vec<Program>, Error> {
    let mut programs: Vec<Program> = Vec::new();
    for (line, lineno) in text.lines().zip(1u32..) {
        match parse_line(line) {
            Err(kind) => return Err(Error { kind, lineno }),
            Ok(None) => (),
            Ok(Some(program)) => programs.push(program),
        }
    }
    Ok(programs)
}

/// Error reading a spec.
#[derive(Debug)]
pub enum ReadError {
    IO(io::Error),
    Parse(Error),
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ReadError::IO(e) => write!(f, "could not read file: {}", e),
            ReadError::Parse(e) => write!(f, "line {}: {}", e.lineno, e.kind),
        }
    }
}

impl From<Error> for ReadError {
    fn from(value: Error) -> Self {
        ReadError::Parse(value)
    }
}

impl From<io::Error> for ReadError {
    fn from(value: io::Error) -> Self {
        ReadError::IO(value)
    }
}

impl error::Error for ReadError {}

/// Read program specs from a file.
pub fn read_spec(path: &Path) -> Result<Vec<Program>, ReadError> {
    let text = fs::read_to_string(path)?;
    Ok(parse_spec(&text)?)
}
