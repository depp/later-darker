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
    let progs = parse::read_spec(&args.spec)?;
    for prog in progs.iter() {
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
