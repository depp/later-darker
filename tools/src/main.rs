use std::error::Error;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process;

use clap::Parser;

mod emit;
mod intern;
mod shader;

#[derive(Parser, Debug)]
struct Args {
    spec: PathBuf,

    output: Option<PathBuf>,

    #[arg(long)]
    dump: bool,
}

fn write(path: &Path, contents: &[u8]) -> io::Result<()> {
    eprintln!("Writing file: {:?}", path);
    fs::write(path, contents)
}

fn run(args: &Args) -> Result<(), Box<dyn Error>> {
    // Read the spec.
    let spec = shader::Spec::read_file(&args.spec)?;
    if args.dump {
        io::stderr().write_all(spec.dump().as_bytes())?;
    }

    // Organize as a list of shaders.
    let manifest = spec.to_manifest();
    if args.dump {
        io::stderr().write_all(manifest.dump().as_bytes())?;
    }

    // Read the shader source code.
    let directory = args.spec.parent().expect("Must have parent directory.");
    let data = shader::Data::read_raw(&manifest, directory)?;

    // Emit the output.
    let output = data.emit_text()?;

    match &args.output {
        None => io::stdout().write_all(output.as_bytes())?,
        Some(path) => write(path, output.as_bytes())?,
    }

    Ok(())
}

fn main() {
    let args = Args::parse();
    if let Err(e) = run(&args) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
