use super::shader::Shader;
use crate::emit::{Error, StringWriter};
use std::fmt::Write;

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
