use crate::xmlgen::{Element, XML};
use arcstr::{ArcStr, literal};
use std::collections::HashMap;

/// A set of Visual Studio project properties.
#[derive(Debug, Clone)]
pub struct PropertyMap(HashMap<ArcStr, Option<ArcStr>>);

impl<K, V> Extend<(K, V)> for PropertyMap
where
    K: Into<ArcStr>,
    V: Into<ArcStr>,
{
    fn extend<T: IntoIterator<Item = (K, V)>>(&mut self, iter: T) {
        for (k, v) in iter.into_iter() {
            self.0.insert(k.into(), Some(v.into()));
        }
    }
}

impl PropertyMap {
    fn from_iter<T, K, V>(iter: T) -> Self
    where
        T: IntoIterator<Item = (K, V)>,
        K: Into<ArcStr>,
        V: Into<ArcStr>,
    {
        let mut map = PropertyMap::new();
        map.extend(iter);
        map
    }

    fn new() -> Self {
        PropertyMap(HashMap::new())
    }

    /// Set a property value.
    pub fn set(&mut self, name: impl Into<ArcStr>, value: impl Into<ArcStr>) {
        self.0.insert(name.into(), Some(value.into()));
    }

    /// Delete a property value.
    #[allow(dead_code)]
    pub fn delete(&mut self, name: impl Into<ArcStr>) {
        self.0.insert(name.into(), None);
    }

    /// Inherit values from another property map.
    fn inherit(&mut self, other: &Self) {
        for (k, v) in other.0.iter() {
            if !self.0.contains_key(k) {
                self.0.insert(k.clone(), v.clone());
            }
        }
    }

    /// Flatten to a sorted list of values.
    fn flatten(&self) -> Vec<(&str, &str)> {
        let mut result = Vec::new();
        for (k, v) in self.0.iter() {
            if let Some(v) = v {
                result.push((k.as_str(), v.as_str()));
            }
        }
        result.sort_by_key(|(k, _)| *k);
        result
    }

    /// Convert preprocessor definitions to a string property.
    fn definition_property(&self) -> Option<String> {
        let mut s = String::new();
        for (k, v) in self.flatten() {
            if !s.is_empty() {
                s.push(';');
            }
            s.push_str(k);
            if v != "1" {
                s.push('=');
                s.push_str(v);
            }
        }
        if s.is_empty() {
            None
        } else {
            s.push_str(";%(PreprocessorDefinitions)");
            Some(s)
        }
    }

    /// Write the property map as XML data.
    fn write_xml(&self, element: &mut Element) {
        for (k, v) in self.flatten() {
            element.tag(k).text(v);
        }
    }
}

/// Visual Studio project configuration properties.
#[derive(Debug, Clone)]
pub struct Properties {
    pub properties: PropertyMap,
    pub cl_compile: PropertyMap,
    pub link: PropertyMap,
    pub definitions: PropertyMap,
}

impl Properties {
    pub fn new() -> Self {
        Properties {
            properties: PropertyMap::new(),
            cl_compile: PropertyMap::new(),
            link: PropertyMap::new(),
            definitions: PropertyMap::new(),
        }
    }

    fn base() -> Self {
        Properties {
            properties: PropertyMap::from_iter([
                ("ConfigurationType", "Application"),
                ("PlatformToolset", "v143"),
                ("CharacterSet", "Unicode"),
            ]),
            cl_compile: PropertyMap::from_iter([
                ("WarningLevel", "Level3"),
                ("SDLCheck", "true"),
                ("ConformanceMode", "true"),
            ]),
            link: PropertyMap::from_iter([
                ("SubSystem", "Windows"),
                ("GenerateDebugInformation", "true"),
            ]),
            definitions: PropertyMap::new(),
        }
    }

    fn debug() -> Self {
        Properties {
            properties: PropertyMap::from_iter([("UseDebugLibraries", "true")]),
            cl_compile: PropertyMap::new(),
            link: PropertyMap::new(),
            definitions: PropertyMap::from_iter([("_DEBUG", "1")]),
        }
    }

    fn release() -> Self {
        Properties {
            properties: PropertyMap::from_iter([
                ("UseDebugLibraries", "false"),
                ("WholeProgramOptimization", "false"),
            ]),
            cl_compile: PropertyMap::from_iter([
                ("FunctionLevelLinking", "true"),
                ("IntrinsicFunctions", "true"),
            ]),
            link: PropertyMap::from_iter([
                ("EnableCOMDATFolding", "true"),
                ("OptimizeReferences", "true"),
            ]),
            definitions: PropertyMap::from_iter([("NDEBUG", "1")]),
        }
    }

    /// Inherit values from another set of properties.
    fn inherit(&mut self, other: &Self) {
        self.properties.inherit(&other.properties);
        self.cl_compile.inherit(&other.cl_compile);
        self.link.inherit(&other.link);
        self.definitions.inherit(&other.definitions);
    }

    /// Resolve properties which are derived from other parts of the structure.
    fn resolve(&mut self) {
        if let Some(value) = self.definitions.definition_property() {
            self.cl_compile.set("PreprocessorDefinitions", value);
        }
    }
}

fn add_files(project: &mut Element, tag: &str, files: &FileList) {
    let mut group = project.tag("ItemGroup").open();
    for file in files.iter() {
        group.tag(tag).attr("Include", file).close();
    }
    group.close();
}

/// A project configuration.
#[derive(Debug, Clone)]
pub struct Configuration {
    pub name: ArcStr,
    pub properties: Properties,
}

/// List of all supported platforms.
const PLATFORMS: [&str; 2] = ["Win32", "x64"];

/// A list of files.
pub type FileList = Vec<ArcStr>;

/// Visual Studio project specification.
#[derive(Debug)]
pub struct Project {
    pub guid: uuid::Uuid,
    pub root_namespace: Option<ArcStr>,
    pub properties: Properties,
    pub configurations: Vec<Configuration>,
    pub cl_include: FileList,
    pub cl_compile: FileList,
    pub resource_compile: FileList,
    pub image: FileList,
}

/// Platform and configuration combination.
struct PlatformConfig {
    platform: ArcStr,
    config: ArcStr,
    condition: String,
    properties: Properties,
}

impl Project {
    pub fn new(guid: uuid::Uuid) -> Self {
        Project {
            guid,
            root_namespace: None,
            properties: Properties::new(),
            configurations: vec![
                Configuration {
                    name: literal!("Debug"),
                    properties: Properties::new(),
                },
                Configuration {
                    name: literal!("Release"),
                    properties: Properties::new(),
                },
            ],
            cl_include: Vec::new(),
            cl_compile: Vec::new(),
            resource_compile: Vec::new(),
            image: Vec::new(),
        }
    }

    fn platform_configs(&self) -> Vec<PlatformConfig> {
        let mut result = Vec::with_capacity(PLATFORMS.len() * self.configurations.len());
        let base = Properties::base();
        let debug = Properties::debug();
        let release = Properties::release();
        for &platform in PLATFORMS.iter() {
            let platform = ArcStr::from(platform);
            for config in self.configurations.iter() {
                let mut properties = config.properties.clone();
                properties.inherit(&self.properties);
                match config.name.as_str() {
                    "Debug" => properties.inherit(&debug),
                    "Release" => properties.inherit(&release),
                    _ => (),
                }
                properties.inherit(&base);
                properties.resolve();
                result.push(PlatformConfig {
                    platform: platform.clone(),
                    config: config.name.clone(),
                    condition: format!(
                        "'$(Configuration)|$(Platform)'=='{}|{}'",
                        config.name, platform
                    ),
                    properties,
                })
            }
        }
        result
    }

    /// Generate the XML vcxproj file.
    pub fn vcxproj(&self) -> String {
        let platform_configs = self.platform_configs();

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
        for config in platform_configs.iter() {
            let mut item = group
                .tag("ProjectConfiguration")
                .attr("Include", format!("{}|{}", config.config, config.platform))
                .open();
            item.tag("Configuration").text(&config.config);
            item.tag("Platform").text(&config.platform);
            item.close();
        }
        group.close();

        // Globals.
        let mut group = project.tag("PropertyGroup").attr("Label", "Globals").open();
        group.tag("VCProjectVersion").text("17.0");
        group.tag("Keyword").text("Win32Proj");
        group
            .tag("ProjectGuid")
            .text(self.guid.braced().to_string());
        if let Some(namespace) = &self.root_namespace {
            group.tag("RootNamespace").text(namespace);
        }
        group.tag("WindowsTargetPlatformVersion").text("10.0");
        group.close();

        // Import default props.
        project
            .tag("Import")
            .attr("Project", "$(VCTargetsPath)\\Microsoft.Cpp.Default.props")
            .close();

        // Configurations.
        for config in platform_configs.iter() {
            let mut group = project
                .tag("PropertyGroup")
                .attr("Condition", &config.condition)
                .attr("Label", "Configuration")
                .open();
            config.properties.properties.write_xml(&mut group);
            group.close();
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
        for config in platform_configs.iter() {
            let mut group = project
                .tag("ImportGroup")
                .attr("Label", "PropertySheets")
                .attr("Condition", &config.condition)
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

        // User macros.
        project
            .tag("PropertyGroup")
            .attr("Label", "UserMacros")
            .close();

        // Compile and link settings.
        for config in platform_configs.iter() {
            let mut group = project
                .tag("ItemDefinitionGroup")
                .attr("Condition", &config.condition)
                .open();
            for (name, properties) in [
                ("ClCompile", &config.properties.cl_compile),
                ("Link", &config.properties.link),
            ] {
                let mut item = group.tag(name).open();
                properties.write_xml(&mut item);
                item.close();
            }
            group.close();
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
