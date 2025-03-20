use crate::emit;
use crate::project::visualstudio::Project;
use arcstr::{ArcStr, literal};
use clap::Parser;
use std::error::Error;
use std::path::PathBuf;
use uuid::uuid;

/// Generate Visual Studio projects.
#[derive(Parser, Debug)]
pub struct Args {
    output_project: PathBuf,
}

fn make_files(files: &[&'static str]) -> Vec<ArcStr> {
    let mut list: Vec<ArcStr> = Vec::with_capacity(files.len());
    list.extend(files.iter().cloned().map(ArcStr::from));
    list
}

impl Args {
    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        let mut project = Project::new(uuid!("26443e89-4e15-4714-8cec-8ce4b3902761"));
        project.root_namespace = Some(literal!("LaterDarker"));
        project.cl_include =
            make_files(&["framework.h", "LaterDarker.h", "Resource.h", "targetver.h"]);
        project.cl_compile = make_files(&["LaterDarker.cpp"]);
        project.resource_compile = make_files(&["LaterDarker.rc"]);
        project.image = make_files(&["LaterDarker.ico", "small.ico"]);

        let data = project.vcxproj();
        emit::write(&self.output_project, data.as_bytes())?;
        Ok(())
    }
}
