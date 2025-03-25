use crate::project::buildinfo::BuildInfo;
use crate::project::paths::ProjectRoot;
use clap::Parser;
use std::error;
use std::io;
use std::io::Write;
use std::path::PathBuf;

/// Build the project.
#[derive(Parser, Debug)]
pub struct Args {
    /// Path to the project root directory.
    #[arg(long)]
    project_directory: Option<PathBuf>,
}

impl Args {
    pub fn run(&self) -> Result<(), Box<dyn error::Error>> {
        let root = ProjectRoot::find_or(self.project_directory.as_deref())?;

        let info = BuildInfo::query(&root)?;

        let mut text = serde_json::to_string_pretty(&info)?;
        text.push('\n');
        io::stdout().write_all(text.as_bytes())?;

        Ok(())
    }
}
