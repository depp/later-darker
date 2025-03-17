use std::error::Error;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process;

use clap::Parser;

mod emit;
mod intern;
mod parse;
mod shader;
mod spec;

#[derive(Parser, Debug)]
struct Args {
    spec: PathBuf,

    #[arg(long)]
    dump: bool,
}

fn run(args: &Args) -> Result<(), Box<dyn Error>> {
    let spec = parse::read_spec(&args.spec)?;
    if args.dump {
        io::stderr().write_all(spec.dump().as_bytes())?;
    }

    let manifest = spec.to_manifest();
    if args.dump {
        io::stderr().write_all(manifest.dump().as_bytes())?;
    }

    let directory = args.spec.parent().expect("Must have parent directory.");
    let mut shaders = Vec::with_capacity(manifest.shaders.len());
    for shader in manifest.shaders.iter() {
        let mut path = PathBuf::from(directory);
        path.push(Path::new(shader.name.as_ref()));
        shaders.push(shader::Shader::read(&path)?);
    }
    let mut output = String::new();
    emit::emit_shaders(&mut output, &shaders)?;
    io::stdout().write_all(output.as_bytes())?;

    Ok(())
}

fn main() {
    let args = Args::parse();
    if let Err(e) = run(&args) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
