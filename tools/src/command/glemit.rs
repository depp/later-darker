use crate::{emit, gl};
use clap::Parser;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
pub struct Args {
    entry_points: PathBuf,

    output_header: Option<PathBuf>,
    output_data: Option<PathBuf>,
}

impl Args {
    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        let text = fs::read_to_string(&self.entry_points)?;
        let entry_points: Vec<&str> = text.lines().collect();
        let api = gl::API::generate(&entry_points)?;
        emit::write_or_stdout(self.output_header.as_deref(), api.header.as_bytes())?;
        emit::write_or_stdout(self.output_data.as_deref(), api.data.as_bytes())?;
        Ok(())
    }
}
