use std::error::Error;
use std::io::{self, Write};
use std::path::PathBuf;

use clap::Parser;

use crate::emit;
use crate::shader;

/// Bundle OpenGL shaders as C++ code.
#[derive(Parser, Debug)]
pub struct Args {
    /// Path to shader spec file.
    spec: PathBuf,

    /// Output C++ file for shader data.
    output: Option<PathBuf>,

    /// Dump internal information about parsed shaders.
    #[arg(long)]
    dump: bool,
}

impl Args {
    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        // Read the spec.
        let spec = shader::Spec::read_file(&self.spec)?;
        if self.dump {
            io::stderr().write_all(spec.dump().as_bytes())?;
        }

        // Organize as a list of shaders.
        let manifest = spec.to_manifest();
        if self.dump {
            io::stderr().write_all(manifest.dump().as_bytes())?;
        }

        // Read the shader source code.
        let directory = self.spec.parent().expect("Must have parent directory.");
        let data = shader::Data::read_raw(&manifest, directory)?;

        // Emit the output.
        let output = data.emit_text()?;

        emit::write_or_stdout(self.output.as_deref(), output.as_bytes())?;
        Ok(())
    }
}
