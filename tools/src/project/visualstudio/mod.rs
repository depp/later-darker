use super::config::Variant;
use super::paths::ProjectRoot;
use super::sources::{SourceList, SourceType};
use crate::emit;
use arcstr::literal;
use project::Project;
use std::error;
use uuid::{Uuid, uuid};

mod project;

struct Parameters {
    name: &'static str,
    guid: Uuid,
}

const FULL: Parameters = Parameters {
    name: "LaterDarkerFull",
    guid: uuid!("26443e89-4e15-4714-8cec-8ce4b3902761"),
};

pub fn generate(
    variant: Variant,
    outputs: &mut emit::Outputs,
    sources: &SourceList,
    root: &ProjectRoot,
) -> Result<(), Box<dyn error::Error>> {
    let parameters = match variant {
        Variant::Compo => panic!("not implemented"),
        Variant::Full => &FULL,
    };
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
    for file in sources.sources().iter() {
        let list = match file.ty() {
            SourceType::Source => &mut project.cl_compile,
            SourceType::Header => &mut project.cl_include,
        };
        list.push(file.path().clone());
    }

    project.emit(outputs, root.as_path(), parameters.name);
    Ok(())
}
