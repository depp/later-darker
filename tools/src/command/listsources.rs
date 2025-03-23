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
        let source_list = sources::SourceList::read_project(&project_directory)?;
        let sources = match &self.config {
            None => source_list.all_sources(),
            Some(config) => {
                let sources = source_list.sources_for_config(&config)?;
                eprintln!(
                    "Config: {} / {} sources",
                    sources.len(),
                    source_list.count()
                );
                sources
            }
        };

        let mut out = String::new();
        for src in sources.iter() {
            out.push_str(src.path().as_str());
            out.push('\n');
        }
        io::stdout().write(out.as_bytes())?;
        Ok(())
    }
}
