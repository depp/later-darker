pub mod glemit;
pub mod glscan;
pub mod listsources;
pub mod shader;
pub mod vsgen;

use std::error::Error;

use clap::Parser;

#[derive(Parser, Debug)]
pub enum Command {
    Shader(shader::Args),
    GLScan(glscan::Args),
    GLEmit(glemit::Args),
    VSGen(vsgen::Args),
    ListSources(listsources::Args),
}

impl Command {
    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        use Command::*;
        match self {
            Shader(c) => c.run(),
            GLScan(c) => c.run(),
            GLEmit(c) => c.run(),
            VSGen(c) => c.run(),
            ListSources(c) => c.run(),
        }
    }
}
