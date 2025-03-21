use crate::emit;
use crate::gl;
use crate::project::config::Config;
use crate::project::config::Platform;
use crate::project::config::Variant;
use crate::project::paths::ProjectPath;
use crate::project::paths::ProjectRoot;
use crate::project::sources;
use crate::project::sources::Source;
use crate::project::sources::SourceList;
use crate::project::sources::SourceType;
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

const GL_API_FULL: &str = "gl_api_full";

impl Args {
    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        let root = ProjectRoot::find_or(self.project_directory.as_deref())?;
        let source_files = SourceList::scan(&root)?;
        let mut outputs = emit::Outputs::new();

        let mut source_files = source_files.filter(&Config {
            platform: Platform::Windows,
            variant: Variant::Full,
        })?;
        let gl_header = Source::new_generated(GL_API_FULL, SourceType::Header)?;
        let gl_source = Source::new_generated(GL_API_FULL, SourceType::Source)?;
        source_files
            .sources
            .extend_from_slice(&[gl_header.clone(), gl_source.clone()]);
        source_files.sort();
        let mut project = Project::new(uuid!("26443e89-4e15-4714-8cec-8ce4b3902761"));
        project.root_namespace = Some(literal!("demo"));
        project
            .properties
            .cl_compile
            .set("LanguageStandard", "stdcpp20");
        project.properties.cl_compile.set(
            "AdditionalIncludeDirectories",
            "$(ProjectDir)src\\generated\\;%(AdditionalIncludeDirectories)",
        );
        project.enable_vcpkg = true;
        for file in source_files.sources.iter() {
            let list = match file.ty() {
                sources::SourceType::Source => &mut project.cl_compile,
                sources::SourceType::Header => &mut project.cl_include,
            };
            list.push(file.path().clone());
        }

        project.emit(&mut outputs, root.as_path(), "LaterDarker");

        let api = gl::API::generate(None)?;
        outputs.add_directory(root.resolve(&ProjectPath::GENERATED));
        outputs.add_file(root.resolve(&gl_header.path()), api.header);
        outputs.add_file(root.resolve(&gl_source.path()), api.data);

        outputs.write()?;
        Ok(())
    }
}
