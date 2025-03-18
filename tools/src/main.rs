use std::process;

use clap::Parser;

mod command;
mod emit;
mod intern;
mod shader;

fn main() {
    let args = command::shader::Args::parse();
    if let Err(e) = args.run() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
