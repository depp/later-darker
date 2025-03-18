use std::fs;
use std::io;
use std::path::Path;

/// A GLSL shader.
pub struct Shader {
    pub text: String,
}

impl Shader {
    pub fn read(path: &Path) -> Result<Self, io::Error> {
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
