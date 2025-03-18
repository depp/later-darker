use std::error::Error;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use clap::Parser;

use crate::shader;

#[derive(Parser, Debug)]
pub struct Args {
    spec: PathBuf,

    output: Option<PathBuf>,

    #[arg(long)]
    dump: bool,
}

fn write(path: &Path, contents: &[u8]) -> io::Result<()> {
    eprintln!("Writing file: {:?}", path);
    fs::write(path, contents)
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

        match &self.output {
            None => io::stdout().write_all(output.as_bytes())?,
            Some(path) => write(path, output.as_bytes())?,
        }

        Ok(())
    }
}
