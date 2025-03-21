use crate::project::paths;
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
        let project_directory =
            paths::find_project_directory_or(self.project_directory.as_deref())?;
        let source_files = sources::SourceList::scan(&project_directory)?;

        let mut project = Project::new(uuid!("26443e89-4e15-4714-8cec-8ce4b3902761"));
        project.root_namespace = Some(literal!("demo"));
        for file in source_files.sources.iter() {
            let list = match file.ty {
                sources::SourceType::Source => &mut project.cl_compile,
                sources::SourceType::Header => &mut project.cl_include,
            };
            let path = file.path.replace('/', "\\");
            list.push(path.into());
        }

        project.emit(&project_directory, "LaterDarker")?;
        Ok(())
    }
}
