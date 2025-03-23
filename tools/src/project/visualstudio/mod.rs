use super::config::{Config, Platform, Variant};
use super::paths::{ProjectPath, ProjectRoot};
use super::sources::{Source, SourceList, SourceType};
use crate::{emit, gl, shader};
use arcstr::literal;
use project::Project;
use std::error;
use uuid::{Uuid, uuid};

mod project;

struct Parameters {
    gl_api: &'static str,
    name: &'static str,
    guid: Uuid,
}

const FULL: Parameters = Parameters {
    gl_api: "gl_api_full",
    name: "LaterCompoFull",
    guid: uuid!("26443e89-4e15-4714-8cec-8ce4b3902761"),
};

pub fn generate(
    variant: Variant,
    outputs: &mut emit::Outputs,
    source_list: SourceList,
    root: &ProjectRoot,
) -> Result<(), Box<dyn error::Error>> {
    let parameters = match variant {
        Variant::Compo => panic!("not implemented"),
        Variant::Full => &FULL,
    };
    let mut source_files = source_list.sources_for_config(&Config {
        platform: Platform::Windows,
        variant: Variant::Full,
    })?;
    let gl_header = Source::new_generated(parameters.gl_api, SourceType::Header)?;
    let gl_source = Source::new_generated(parameters.gl_api, SourceType::Source)?;
    let shader_data = Source::new_generated("gl_shader_text_full", SourceType::Source)?;
    source_files.extend_from_slice(&[gl_header.clone(), gl_source.clone(), shader_data.clone()]);
    let mut project = Project::new(parameters.guid);
    project
        .property_sheets
        .push(literal!("support\\Common.props"));
    project.root_namespace = Some(literal!("demo"));
    project.properties.link.set(
        "AdditionalDependencies",
        "opengl32.lib;%(AdditionalDependencies)",
    );
    project.enable_vcpkg = true;
    for file in source_files.iter() {
        let list = match file.ty() {
            SourceType::Source => &mut project.cl_compile,
            SourceType::Header => &mut project.cl_include,
        };
        list.push(file.path().clone());
    }

    project.emit(outputs, root.as_path(), parameters.name);

    // Generate source fiels.
    outputs.add_directory(root.resolve(&ProjectPath::GENERATED));

    // Generate OpenGL bindings.
    let api = gl::API::create()?.make_bindings();
    outputs.add_file(root.resolve(&gl_header.path()), api.header);
    outputs.add_file(root.resolve(&gl_source.path()), api.data);

    // Generate shader bundle.
    let shader_dir = &ProjectPath::SHADER;
    let manifest =
        shader::Spec::read_file(&root.resolve(&shader_dir.append("shaders.txt")?))?.to_manifest();
    let data = shader::Data::read_raw(&manifest, &root.resolve(&shader_dir))?;
    outputs.add_file(root.resolve(shader_data.path()), data.emit_text()?);

    Ok(())
}
