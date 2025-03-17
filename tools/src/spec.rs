use std::path::Path;
use std::{error, fmt, fs, io};

/// A spec for a shader program to compile and link.
#[derive(Debug, Clone)]
pub struct Program {
    /// Program name. Used for variable names in the generated source code.
    pub name: String,
    /// Vertex shader source filename.
    pub vertex: String,
    /// Fragment shader source filename.
    pub fragment: String,
}

/// A type of shader.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShaderType {
    Vertex,
    Fragment,
}

impl ShaderType {
    /// Get the type of a shader from its file extension.
    fn from_extension(ext: &str) -> Option<Self> {
        Some(match ext {
            "vert" => ShaderType::Vertex,
            "frag" => ShaderType::Fragment,
            _ => return None,
        })
    }
}

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
pub struct ParseError {
    kind: ErrorKind,
    lineno: u32,
}

/// Error parsing or reading a spec.
#[derive(Debug)]
pub enum Error {
    IO(io::Error),
    Parse(ParseError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::IO(e) => write!(f, "could not read file: {}", e),
            Error::Parse(e) => write!(f, "line {}: {}", e.lineno, e.kind),
        }
    }
}

impl From<ParseError> for Error {
    fn from(value: ParseError) -> Self {
        Error::Parse(value)
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Error::IO(value)
    }
}

impl error::Error for Error {}

impl Program {
    fn parse_line(line: &str) -> Result<Option<Self>, ErrorKind> {
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

    fn parse(text: &str) -> Result<Vec<Self>, ParseError> {
        let mut programs: Vec<Program> = Vec::new();
        for (line, lineno) in text.lines().zip(1u32..) {
            match Program::parse_line(line) {
                Err(kind) => return Err(ParseError { kind, lineno }),
                Ok(None) => (),
                Ok(Some(program)) => programs.push(program),
            }
        }
        Ok(programs)
    }

    pub fn read(path: &Path) -> Result<Vec<Program>, Error> {
        let text = fs::read_to_string(path)?;
        Ok(Program::parse(&text)?)
    }
}
