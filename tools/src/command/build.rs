use crate::project::config::{Config, Platform, Variant};
use crate::project::paths::{ProjectPath, ProjectRoot};
use crate::project::sources::{GeneratorSet, SourceSpec};
use crate::project::visualstudio;
use crate::{emit, vsenv};
use clap::Parser;
use std::error::Error;
use std::path::PathBuf;
use std::process::Command;

/// Build the project.
#[derive(Parser, Debug)]
pub struct Args {
    #[arg(long)]
    project_directory: Option<PathBuf>,
}

impl Args {
    /// Generate sources. Returns a list of project files.
    fn generate_sources(
        &self,
        root: &ProjectRoot,
    ) -> Result<Vec<visualstudio::ProjectInfo>, Box<dyn Error>> {
        let source_spec = SourceSpec::read_project(&root)?;
        let mut outputs = emit::Outputs::new();
        let mut generators = GeneratorSet::new();
        let mut projects = Vec::new();

        for variant in [Variant::Full, Variant::Compo] {
            let sources = source_spec.sources_for_config(&Config {
                platform: Platform::Windows,
                variant,
            })?;
            projects.push(visualstudio::generate(
                variant,
                &mut outputs,
                &sources,
                &root,
            )?);
            generators.add(&sources);
        }

        outputs.add_directory(root.resolve(&ProjectPath::GENERATED));
        generators.run(&root, &source_spec, &mut outputs)?;
        outputs.write()?;

        Ok(projects)
    }

    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        let root = ProjectRoot::find_or(self.project_directory.as_deref())?;
        eprintln!("Project root: {}", root.as_path().display());
        let msbuild = vsenv::find_msbuild()?;
        eprintln!("MSBuild: {}", msbuild);

        let projects = self.generate_sources(&root)?;
        for platform in ["Win32", "x64"] {
            for project in projects.iter() {
                let status = Command::new(&msbuild)
                    .current_dir(root.as_path())
                    .arg(&project.project_name)
                    .arg("-property:Configuration=Release")
                    .arg(format!("-property:Platform={}", platform))
                    .arg("-maxCpuCount") // Uses all available CPUs.
                    .status();
                eprintln!("Status: {:?}", status);
            }
        }

        Ok(())
    }
}
