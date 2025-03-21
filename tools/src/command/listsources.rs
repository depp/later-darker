use crate::project::config::Config;
use crate::project::paths;
use crate::project::sources;
use clap::Parser;
use std::error::Error;
use std::io::{self, Write};
use std::path::PathBuf;

/// List all source files.
#[derive(Parser, Debug)]
pub struct Args {
    /// Path to the project directory.
    #[arg(long)]
    project_directory: Option<PathBuf>,

    #[arg(long)]
    config: Option<Config>,
}

impl Args {
    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        let project_directory = paths::ProjectRoot::find_or(self.project_directory.as_deref())?;
        let mut source_files = sources::SourceList::scan(&project_directory)?;
        if let Some(config) = &self.config {
            let n = source_files.sources.len();
            source_files = source_files.filter(config)?;
            let m = source_files.sources.len();
            eprintln!("Config: {} / {} sources", m, n);
        }
        source_files.sort();

        let mut out = String::new();
        for src in source_files.sources.iter() {
            out.push_str(src.path().as_str());
            if let Some(expr) = src.build_tag() {
                out.push(' ');
                out.push_str(&expr.to_string());
            }
            out.push('\n');
        }
        io::stdout().write(out.as_bytes())?;
        Ok(())
    }
}
