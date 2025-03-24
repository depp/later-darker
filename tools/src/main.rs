use std::process;

use clap::Parser;

mod command;
mod emit;
mod error;
mod gl;
mod identifier;
mod intern;
mod project;
mod shader;
mod vsenv;
mod xmlgen;
mod xmlparse;

fn main() {
    let cmd = command::Command::parse();
    if let Err(e) = cmd.run() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
