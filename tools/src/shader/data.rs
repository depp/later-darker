use super::spec::Manifest;
use crate::emit;
use std::error;
use std::fmt::{self, Write};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Code generation error.
#[derive(Debug, Clone, Copy)]
pub enum EmitError {
    NullByte,
}

impl fmt::Display for EmitError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &EmitError::NullByte => f.write_str("shader source code contains null byte"),
        }
    }
}

impl error::Error for EmitError {}

/// An individual shader.
#[derive(Debug, Clone)]
pub struct Shader {
    text: String,
}

impl Shader {
    /// Read shader source code from a file.
    pub fn read_raw(path: &Path) -> Result<Self, io::Error> {
        let raw_text = fs::read_to_string(path)?;
        let mut text = String::with_capacity(raw_text.len() + 1);
        for line in raw_text.lines() {
            text.push_str(line.trim_ascii_end());
            text.push('\n');
        }
        text.truncate(text.trim_ascii_end().len());
        Ok(Shader { text })
    }
}

/// Collection of shader data that can be embedded in the ddemo.
#[derive(Debug, Clone)]
pub struct Data {
    shaders: Vec<Shader>,
}

impl Data {
    /// Read raw shader data.
    pub fn read_raw(manifest: &Manifest, directory: &Path) -> io::Result<Self> {
        let mut shaders = Vec::with_capacity(manifest.shaders.len());
        for shader in manifest.shaders.iter() {
            let mut path = PathBuf::from(directory);
            path.push(Path::new(shader.name.as_ref()));
            shaders.push(Shader::read_raw(&path)?);
        }
        Ok(Data { shaders })
    }

    pub fn emit_text(&self) -> Result<String, EmitError> {
        // Null bytes are used to separate shaders, so they cannot be in the
        // shader sources.
        if self.shaders.iter().any(|s| s.text.contains('\0')) {
            return Err(EmitError::NullByte);
        }

        // Get size, including null bytes.
        let size: usize =
            self.shaders.iter().map(|s| s.text.len()).sum::<usize>() + self.shaders.len();

        let mut output = String::new();
        // Header.
        output.push_str(emit::HEADER);
        output.push_str("namespace demo {\nnamespace gl_shader {\n");

        // Shader text.
        write!(output, "extern const char ShaderText[{}] =\n", size).unwrap();
        let mut writer = emit::StringWriter::new(&mut output);
        for (n, shader) in self.shaders.iter().enumerate() {
            if n != 0 {
                writer.write(&[0]);
            }
            writer.write(shader.text.as_bytes());
        }
        writer.finish();
        output.push_str(";\n");

        // Footer.
        output.push_str("}\n}\n");

        Ok(output)
    }
}
