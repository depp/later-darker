use crate::emit;
use crate::project::visualstudio::Project;
use clap::Parser;
use std::error::Error;
use std::path::PathBuf;
use std::sync::Arc;

/// Generate Visual Studio projects.
#[derive(Parser, Debug)]
pub struct Args {
    output_project: PathBuf,
}

fn make_files(files: &[&str]) -> Vec<Arc<str>> {
    let mut list: Vec<Arc<str>> = Vec::with_capacity(files.len());
    list.extend(files.iter().cloned().map(Arc::from));
    list
}

impl Args {
    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        let project = Project {
            guid: "{26443e89-4e15-4714-8cec-8ce4b3902761}".to_string(),
            root_namespace: "LaterDarker".to_string(),
            cl_include: make_files(&["framework.h", "LaterDarker.h", "Resource.h", "targetver.h"]),
            cl_compile: make_files(&["LaterDarker.cpp"]),
            resource_compile: make_files(&["LaterDarker.rc"]),
            image: make_files(&["LaterDarker.ico", "small.ico"]),
        };
        let data = project.vcxproj();
        emit::write(&self.output_project, data.as_bytes())?;
        Ok(())
    }
}
