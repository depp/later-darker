use std::error::Error;
use std::path::PathBuf;
use std::process;

use clap::Parser;

mod parse;
mod spec;

#[derive(Parser, Debug)]
struct Args {
    spec: PathBuf,
}

fn run(args: &Args) -> Result<(), Box<dyn Error>> {
    let spec = parse::read_spec(&args.spec)?;
    for prog in spec.programs.iter() {
        eprintln!("Program: {:?}", prog);
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
