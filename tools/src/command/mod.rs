pub mod glscan;
pub mod shader;

use std::error::Error;

use clap::Parser;

#[derive(Parser, Debug)]
pub enum Command {
    Shader(shader::Args),
    GLScan(glscan::Args),
}

impl Command {
    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        match self {
            Command::Shader(c) => c.run(),
            Command::GLScan(c) => c.run(),
        }
    }
}
