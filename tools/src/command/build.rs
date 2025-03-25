use crate::emit;
use crate::project::config::{Config, Platform, Variant};
use crate::project::paths::{ProjectPath, ProjectRoot};
use crate::project::sources::{GeneratorSet, SourceSpec};
use crate::project::visualstudio;
use crate::vsenv::{self, Arch};
use clap::Parser;
use std::collections::HashSet;
use std::error::{self, Error};
use std::fmt;
use std::io;
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;

/// Build the project.
#[derive(Parser, Debug)]
pub struct Args {
    #[arg(long)]
    project_directory: Option<PathBuf>,

    #[arg(long, value_delimiter = ',')]
    configurations: Option<Vec<Configuration>>,
}

/// A build configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Configuration(Arch, Variant);

impl fmt::Display for Configuration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.0, self.1)
    }
}

const DEFAULT_CONFIGS: [Configuration; 3] = [
    Configuration(Arch::Amd64, Variant::Full),
    Configuration(Arch::Amd64, Variant::Compo),
    Configuration(Arch::X86, Variant::Compo),
];

#[derive(Debug)]
enum ConfigurationParseErr {
    Syntax,
    Arch(String),
    Variant(String),
}

impl fmt::Display for ConfigurationParseErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ConfigurationParseErr::*;
        match self {
            Syntax => f.write_str("invalid configuration syntax, should be arch:variant"),
            Variant(text) => write!(f, "unknown variant {:?}", text),
            Arch(text) => write!(f, "unknown architecture {:?}", text),
        }
    }
}

impl error::Error for ConfigurationParseErr {}

impl FromStr for Configuration {
    type Err = ConfigurationParseErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (arch, variant) = s.split_once(':').ok_or(ConfigurationParseErr::Syntax)?;
        let arch =
            Arch::from_str(arch).map_err(|_| ConfigurationParseErr::Arch(arch.to_string()))?;
        let variant = Variant::from_str(variant)
            .map_err(|_| ConfigurationParseErr::Variant(variant.to_string()))?;
        Ok(Self(arch, variant))
    }
}

impl Args {
    /// Generate sources. Returns a list of project files.
    fn generate_sources(
        &self,
        root: &ProjectRoot,
        variants: &[Variant],
    ) -> Result<Vec<visualstudio::ProjectInfo>, Box<dyn Error>> {
        let source_spec = SourceSpec::read_project(&root)?;
        let mut outputs = emit::Outputs::new();
        let mut generators = GeneratorSet::new();
        let mut projects = Vec::new();

        for &variant in variants.iter() {
            let sources = source_spec.sources_for_config(&Config {
                platform: Platform::Windows,
                variant,
            })?;
            projects.push(visualstudio::generate(
                variant,
                &mut outputs,
                &sources,
                &root,
            )?);
            generators.add(&sources);
        }

        outputs.add_directory(root.resolve(&ProjectPath::GENERATED));
        generators.run(&root, &source_spec, &mut outputs)?;
        outputs.write()?;

        Ok(projects)
    }

    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        let configurations = match &self.configurations {
            None => DEFAULT_CONFIGS.to_vec(),
            Some(value) => dedup(value),
        };
        let (_, variants) = values(&configurations);
        let root = ProjectRoot::find_or(self.project_directory.as_deref())?;
        eprintln!("Project root: {}", root.as_path().display());
        let msbuild = vsenv::find_msbuild()?;
        eprintln!("MSBuild: {}", msbuild);

        let projects = self.generate_sources(&root, &variants)?;
        for &configuration in configurations.iter() {
            let Configuration(arch, variant) = configuration;
            let project = projects
                .iter()
                .find(|p| p.variant == variant)
                .expect("Created earlier");
            match Command::new(&msbuild)
                .current_dir(root.as_path())
                .arg(&project.project_name)
                .arg("-property:Configuration=Release")
                .arg(format!("-property:Platform={}", arch_name(arch)))
                .arg("-maxCpuCount") // Uses all available CPUs.
                .status()
            {
                Ok(status) => {
                    if !status.success() {
                        return Err(BuildFailed(configuration, FailReason::FailStatus).into());
                    }
                }
                Err(err) => return Err(BuildFailed(configuration, FailReason::IO(err)).into()),
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
enum FailReason {
    IO(io::Error),
    FailStatus,
}

impl fmt::Display for FailReason {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FailReason::IO(err) => write!(f, "could not run build: {}", err),
            FailReason::FailStatus => f.write_str("build process failed"),
        }
    }
}

#[derive(Debug)]
struct BuildFailed(Configuration, FailReason);

impl fmt::Display for BuildFailed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "build failed for configuration {}: {}", self.0, self.1)
    }
}

impl error::Error for BuildFailed {}

/// Deduplicate build configurations.
fn dedup(configurations: &[Configuration]) -> Vec<Configuration> {
    let mut present: HashSet<Configuration> = HashSet::with_capacity(configurations.len());
    let mut result = Vec::new();
    for &configuration in configurations.iter() {
        if present.insert(configuration) {
            result.push(configuration);
        }
    }
    result
}

/// List all values the different parts of the configuration take.
fn values(configurations: &[Configuration]) -> (Vec<Arch>, Vec<Variant>) {
    let mut archs = Vec::new();
    let mut variants = Vec::new();
    for &Configuration(arch, variant) in configurations.iter() {
        if !archs.contains(&arch) {
            archs.push(arch);
        }
        if !variants.contains(&variant) {
            variants.push(variant);
        }
    }
    (archs, variants)
}

/// Get the architecture name, as used by MSBuild.
fn arch_name(arch: Arch) -> &'static str {
    use Arch::*;
    match arch {
        X86 => "Win32",
        Amd64 => "x64",
        _ => arch.name(),
    }
}
