use crate::emit;
use crate::project::config::{Config, Platform, Variant};
use crate::project::paths::{ProjectPath, ProjectRoot};
use crate::project::sources::SourceSpec;
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
        let source_spec = SourceSpec::read_project(&root)?;
        let variant = Variant::Full;
        let sources = source_spec.sources_for_config(&Config {
            platform: Platform::Windows,
            variant,
        })?;
        let mut outputs = emit::Outputs::new();
        visualstudio::generate(Variant::Full, &mut outputs, &sources, &root)?;
        outputs.add_directory(root.resolve(&ProjectPath::GENERATED));
        for generator in sources.generators() {
            for output in generator.implementation().run(&root)? {
                outputs.add_file(root.resolve(&output.path), output.data);
            }
        }
        outputs.write()?;
        Ok(())
    }
}
