use crate::gl;
use clap::Parser;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
pub struct Args {
    entry_points: PathBuf,

    output: Option<PathBuf>,
}

impl Args {
    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        let text = fs::read_to_string(&self.entry_points)?;
        let entry_points: Vec<&str> = text.lines().collect();
        gl::generate(&entry_points)?;
        Ok(())
    }
}
