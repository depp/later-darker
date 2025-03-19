use crate::emit;
use crate::gl;
use clap::Parser;
use std::collections::HashSet;
use std::error::Error;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(long)]
    entry_points: Option<PathBuf>,

    #[arg(long)]
    output_header: Option<PathBuf>,
    #[arg(long)]
    output_data: Option<PathBuf>,
}

impl Args {
    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        let entry_points = self
            .entry_points
            .as_deref()
            .map(read_entry_points)
            .transpose()?;
        let api = gl::API::generate(entry_points.as_ref())?;
        emit::write_or_stdout(self.output_header.as_deref(), api.header.as_bytes())?;
        emit::write_or_stdout(self.output_data.as_deref(), api.data.as_bytes())?;
        Ok(())
    }
}

fn read_entry_points(path: &Path) -> io::Result<HashSet<Arc<str>>> {
    let text = fs::read_to_string(path)?;
    let mut result = HashSet::new();
    result.extend(text.lines().map(Arc::from));
    Ok(result)
}
