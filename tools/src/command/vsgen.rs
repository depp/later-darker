use crate::emit;
use crate::project::config::Variant;
use crate::project::paths::ProjectRoot;
use crate::project::sources::SourceList;
use crate::project::visualstudio;
use clap::Parser;
use std::error::Error;
use std::path::PathBuf;

/// Generate Visual Studio projects.
#[derive(Parser, Debug)]
pub struct Args {
    /// Path to the project directory.
    #[arg(long)]
    project_directory: Option<PathBuf>,
}

impl Args {
    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        let root = ProjectRoot::find_or(self.project_directory.as_deref())?;
        let source_list = SourceList::read_project(&root)?;
        let mut outputs = emit::Outputs::new();
        visualstudio::generate(Variant::Full, &mut outputs, source_list, &root)?;
        outputs.write()?;
        Ok(())
    }
}
