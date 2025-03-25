#[cfg(target_os = "windows")]
pub mod build;
pub mod buildinfo;
pub mod glemit;
pub mod glscan;
pub mod listsources;
pub mod shader;
#[cfg(target_os = "windows")]
pub mod vsenv;
pub mod vsgen;

use clap::Parser;
use std::error::Error;

#[derive(Parser, Debug)]
pub enum Command {
    Shader(shader::Args),
    GLScan(glscan::Args),
    GLEmit(glemit::Args),
    VSGen(vsgen::Args),
    ListSources(listsources::Args),
    BuildInfo(buildinfo::Args),

    #[cfg(target_os = "windows")]
    Build(build::Args),
    #[cfg(target_os = "windows")]
    VSEnv(vsenv::Args),
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
            BuildInfo(c) => c.run(),

            #[cfg(target_os = "windows")]
            Build(c) => c.run(),
            #[cfg(target_os = "windows")]
            VSEnv(c) => c.run(),
        }
    }
}
