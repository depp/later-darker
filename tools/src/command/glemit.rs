use crate::emit;
use crate::gl::api;
use clap::Parser;
use std::collections::HashSet;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

/// Generate OpenGL API bindings.
#[derive(Parser, Debug)]
pub struct Args {
    /// File with list of OpenGL functions, one per line.
    #[arg(long)]
    entry_points: Option<PathBuf>,

    /// Output C++ header file.
    #[arg(long)]
    output_header: Option<PathBuf>,

    /// Output C++ source file.
    #[arg(long)]
    output_data: Option<PathBuf>,
}

impl Args {
    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        let api = api::APISpec {
            version: api::Version(3, 3),
            extensions: vec![],
        };
        let link = api::APISpec {
            version: api::Version(1, 1),
            extensions: vec![],
        };
        let api = api::API::create(&api, &link)?;
        let bindings = match &self.entry_points {
            None => api.make_bindings(),
            Some(path) => {
                let text = fs::read_to_string(path)?;
                let mut entry_points = HashSet::new();
                for line in text.lines() {
                    entry_points.insert(line.to_string());
                }
                api.make_subset_bindings(&entry_points)?
            }
        };
        emit::write_or_stdout(self.output_header.as_deref(), bindings.header.as_bytes())?;
        emit::write_or_stdout(self.output_data.as_deref(), bindings.data.as_bytes())?;
        Ok(())
    }
}
