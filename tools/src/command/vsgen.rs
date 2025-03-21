use crate::project::config::Config;
use crate::project::config::Platform;
use crate::project::config::Variant;
use crate::project::paths::ProjectRoot;
use crate::project::sources;
use crate::project::visualstudio::Project;
use arcstr::literal;
use clap::Parser;
use std::error::Error;
use std::path::PathBuf;
use uuid::uuid;

/// Generate Visual Studio projects.
#[derive(Parser, Debug)]
pub struct Args {
    /// Path to the project directory.
    #[arg(long)]
    project_directory: Option<PathBuf>,
}

impl Args {
    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        let project_directory = ProjectRoot::find_or(self.project_directory.as_deref())?;
        let source_files = sources::SourceList::scan(&project_directory)?;

        let source_files = source_files.filter(&Config {
            platform: Platform::Windows,
            variant: Variant::Full,
        })?;
        let mut project = Project::new(uuid!("26443e89-4e15-4714-8cec-8ce4b3902761"));
        project.root_namespace = Some(literal!("demo"));
        project
            .properties
            .cl_compile
            .set("LanguageStandard", "stdcpp20");
        for file in source_files.sources.iter() {
            let list = match file.ty() {
                sources::SourceType::Source => &mut project.cl_compile,
                sources::SourceType::Header => &mut project.cl_include,
            };
            list.push(file.path().to_windows().into());
        }

        project.emit(project_directory.as_path(), "LaterDarker")?;
        Ok(())
    }
}
