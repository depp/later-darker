pub mod build;
pub mod glemit;
pub mod glscan;
pub mod listsources;
pub mod shader;
pub mod vsenv;
pub mod vsgen;

use std::error::Error;

use clap::Parser;

#[derive(Parser, Debug)]
pub enum Command {
    Build(build::Args),
    Shader(shader::Args),
    GLScan(glscan::Args),
    GLEmit(glemit::Args),
    VSEnv(vsenv::Args),
    VSGen(vsgen::Args),
    ListSources(listsources::Args),
}

impl Command {
    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        use Command::*;
        match self {
            Build(c) => c.run(),
            Shader(c) => c.run(),
            GLScan(c) => c.run(),
            GLEmit(c) => c.run(),
            VSEnv(c) => c.run(),
            VSGen(c) => c.run(),
            ListSources(c) => c.run(),
        }
    }
}
