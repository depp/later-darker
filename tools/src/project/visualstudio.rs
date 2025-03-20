use crate::xmlgen::{Element, XML};
use std::sync::Arc;

/// A list of files.
pub type FileList = Vec<Arc<str>>;

/// Visual Studio project specification.
#[derive(Debug)]
pub struct Project {
    pub guid: String,
    pub root_namespace: String,
    pub cl_include: FileList,
    pub cl_compile: FileList,
    pub resource_compile: FileList,
    pub image: FileList,
}

fn add_files(project: &mut Element, tag: &str, files: &FileList) {
    let mut group = project.tag("ItemGroup").open();
    for file in files.iter() {
        group.tag(tag).attr("Include", file).close();
    }
    group.close();
}

pub type PropertyList = &'static [(&'static str, &'static str)];

struct Configuration {
    name: &'static str,
    properties: PropertyList,
    cl_compile: PropertyList,
    link: PropertyList,
}

const PLATFORMS: [&str; 2] = ["Win32", "x64"];
const CONFIGURATIONS: [Configuration; 2] = [
    Configuration {
        name: "Debug",
        properties: &[
            ("ConfigurationType", "Application"),
            ("UseDebugLibraries", "true"),
            ("PlatformToolset", "v143"),
            ("CharacterSet", "Unicode"),
        ],
        cl_compile: &[
            ("WarningLevel", "Level3"),
            ("SDLCheck", "true"),
            (
                "PreprocessorDefinitions",
                "_DEBUG;_WINDOWS;%(PreprocessorDefinitions)",
            ),
            ("ConformanceMode", "true"),
        ],
        link: &[
            ("SubSystem", "Windows"),
            ("GenerateDebugInformation", "true"),
        ],
    },
    Configuration {
        name: "Release",
        properties: &[
            ("ConfigurationType", "Application"),
            ("UseDebugLibraries", "false"),
            ("PlatformToolset", "v143"),
            ("WholeProgramOptimization", "true"),
            ("CharacterSet", "Unicode"),
        ],
        cl_compile: &[
            ("WarningLevel", "Level3"),
            ("FunctionLevelLinking", "true"),
            ("IntrinsicFunctions", "true"),
            ("SDLCheck", "true"),
            (
                "PreprocessorDefinitions",
                "NDEBUG;_WINDOWS;%(PreprocessorDefinitions)",
            ),
            ("ConformanceMode", "true"),
        ],
        link: &[
            ("SubSystem", "Windows"),
            ("EnableCOMDATFolding", "true"),
            ("OptimizeReferences", "true"),
            ("GenerateDebugInformation", "true"),
        ],
    },
];

fn condition(platform: &str, config: &str) -> String {
    format!("'$(Configuration)|$(Platform)'=='{}|{}'", config, platform)
}

impl Project {
    /// Generate the XML vcxproj file.
    pub fn vcxproj(&self) -> String {
        let mut doc = XML::new();
        let mut project = doc
            .root("Project")
            .attr("DefaultTargets", "Build")
            .attr(
                "xmlns",
                "http://schemas.microsoft.com/developer/msbuild/2003",
            )
            .open();

        // Project Configurations
        let mut group = project
            .tag("ItemGroup")
            .attr("Label", "ProjectConfigurations")
            .open();
        for &platform in PLATFORMS.iter() {
            for config in CONFIGURATIONS.iter() {
                let mut item = group
                    .tag("ProjectConfiguration")
                    .attr("Include", &format!("{}|{}", config.name, platform))
                    .open();
                item.tag("Configuration").text(config.name);
                item.tag("Platform").text(platform);
                item.close();
            }
        }
        group.close();

        // Globals.
        let mut group = project.tag("PropertyGroup").attr("Label", "Globals").open();
        group.tag("VCProjectVersion").text("17.0");
        group.tag("Keyword").text("Win32Proj");
        group.tag("ProjectGuid").text(&self.guid);
        group.tag("RootNamespace").text(&self.root_namespace);
        group.tag("WindowsTargetPlatformVersion").text("10.0");
        group.close();

        // Import default props.
        project
            .tag("Import")
            .attr("Project", "$(VCTargetsPath)\\Microsoft.Cpp.Default.props")
            .close();

        // Configurations.
        for &platform in PLATFORMS.iter() {
            for config in CONFIGURATIONS.iter() {
                let mut group = project
                    .tag("PropertyGroup")
                    .attr("Condition", &condition(platform, config.name))
                    .attr("Label", "Configuration")
                    .open();
                for &(key, value) in config.properties.iter() {
                    group.tag(key).text(value);
                }
                group.close();
            }
        }

        // Global props.
        project
            .tag("Import")
            .attr("Project", "$(VCTargetsPath)\\Microsoft.Cpp.props")
            .close();

        // Extension settings.
        project
            .tag("ImportGroup")
            .attr("Label", "ExtensionSettings")
            .open()
            .close();

        // Shared.
        project
            .tag("ImportGroup")
            .attr("Label", "Shared")
            .open()
            .close();

        // Property sheets.
        for &platform in PLATFORMS.iter() {
            for config in CONFIGURATIONS.iter() {
                let mut group = project
                    .tag("ImportGroup")
                    .attr("Label", "PropertySheets")
                    .attr("Condition", &condition(platform, config.name))
                    .open();
                group
                    .tag("Import")
                    .attr(
                        "Project",
                        "$(UserRootDir)\\Microsoft.Cpp.$(Platform).user.props",
                    )
                    .attr(
                        "Condition",
                        "exists('$(UserRootDir)\\Microsoft.Cpp.$(Platform).user.props')",
                    )
                    .attr("Label", "LocalAppDataPlatform")
                    .close();
                group.close();
            }
        }

        // User macros.
        project
            .tag("PropertyGroup")
            .attr("Label", "UserMacros")
            .close();

        // Compile and link settings.
        for &platform in PLATFORMS.iter() {
            for config in CONFIGURATIONS.iter() {
                let mut group = project
                    .tag("ItemDefinitionGroup")
                    .attr("Condition", &condition(platform, config.name))
                    .open();
                for (name, properties) in [("ClCompile", config.cl_compile), ("Link", config.link)]
                {
                    let mut item = group.tag(name).open();
                    for &(key, value) in properties.iter() {
                        item.tag(key).text(value);
                    }
                    item.close();
                }
                group.close();
            }
        }

        // Files.
        add_files(&mut project, "ClInclude", &self.cl_include);
        add_files(&mut project, "ClCompile", &self.cl_compile);
        add_files(&mut project, "ResourceCompile", &self.resource_compile);
        add_files(&mut project, "Image", &self.image);

        // Targets.
        project
            .tag("Import")
            .attr("Project", "$(VCTargetsPath)\\Microsoft.Cpp.targets")
            .close();
        project
            .tag("ImportGroup")
            .attr("Label", "ExtensionTargets")
            .open()
            .close();

        // End.
        project.close();

        doc.finish()
    }
}
