use std::error::Error;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process;

use clap::Parser;

mod intern;
mod parse;
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

    Ok(())
}

fn main() {
    let args = Args::parse();
    if let Err(e) = run(&args) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
