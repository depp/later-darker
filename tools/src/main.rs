use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    spec: PathBuf,
}

fn main() {
    let args = Args::parse();
    eprintln!("args = {:?}", args);
}
