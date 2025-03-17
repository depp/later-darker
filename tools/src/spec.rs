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
    pub fn from_extension(ext: &str) -> Option<Self> {
        Some(match ext {
            "vert" => ShaderType::Vertex,
            "frag" => ShaderType::Fragment,
            _ => return None,
        })
    }
}
