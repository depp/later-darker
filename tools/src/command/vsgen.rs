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
use crate::shader;
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
        let shader_data = Source::new_generated("gl_shader_text_full", SourceType::Source)?;
        source_files.sources.extend_from_slice(&[
            gl_header.clone(),
            gl_source.clone(),
            shader_data.clone(),
        ]);
        source_files.sort();
        let mut project = Project::new(uuid!("26443e89-4e15-4714-8cec-8ce4b3902761"));
        project
            .property_sheets
            .push(literal!("support\\Common.props"));
        project.root_namespace = Some(literal!("demo"));
        project.properties.link.set(
            "AdditionalDependencies",
            "opengl32.lib;%(AdditionalDependencies)",
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

        // Generate source fiels.
        outputs.add_directory(root.resolve(&ProjectPath::GENERATED));

        // Generate OpenGL bindings.
        let api = gl::API::generate(None)?;
        outputs.add_file(root.resolve(&gl_header.path()), api.header);
        outputs.add_file(root.resolve(&gl_source.path()), api.data);

        // Generate shader bundle.
        let shader_dir = &ProjectPath::SHADER;
        let manifest = shader::Spec::read_file(&root.resolve(&shader_dir.append("shaders.txt")?))?
            .to_manifest();
        let data = shader::Data::read_raw(&manifest, &root.resolve(&shader_dir))?;
        outputs.add_file(root.resolve(shader_data.path()), data.emit_text()?);

        outputs.write()?;
        Ok(())
    }
}
