use super::config::Config;
use super::paths::{ProjectPath, ProjectRoot};
use super::sources::{Source, SourceSpec};
use crate::error::FileError;
use crate::gl::api;
use crate::gl::scan;
use crate::project::sources::SourceType;
use crate::shader;
use std::error;
use std::fmt;
use std::str::FromStr;
use std::sync::Arc;

// ============================================================================
// Errors
// ============================================================================

/// An error constructing a generator rule.
#[derive(Debug)]
pub enum EvaluationError {
    UnknownRule(String),
    UnexpectedOutput(ProjectPath),
    MissingOutput(SourceType),
    UnknownProperty(String),
    MissingProperty(String),
    PropertyValue(String, Box<dyn error::Error>),
}

impl fmt::Display for EvaluationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use EvaluationError::*;
        match self {
            UnknownRule(name) => write!(f, "unknown generator rule: {:?}", name),
            UnexpectedOutput(path) => write!(f, "unexpected output: {}", path),
            MissingOutput(ty) => write!(f, "missing required output with type {:?}", ty),
            UnknownProperty(name) => write!(f, "unknown property: {:?}", name),
            MissingProperty(name) => write!(f, "missing required property: {:?}", name),
            PropertyValue(name, err) => write!(f, "invalid value for {:?} property: {}", name, err),
        }
    }
}

impl error::Error for EvaluationError {}

// ============================================================================
// Generator Interface
// ============================================================================

/// An individual output from a generator.
pub struct Output {
    pub path: ProjectPath,
    pub data: Vec<u8>,
}

pub trait Generator: fmt::Debug {
    /// Run the generator, producing output data.
    fn run(
        &self,
        root: &ProjectRoot,
        sources: &SourceSpec,
    ) -> Result<Vec<Output>, Box<dyn error::Error>>;
}

/// Construct a generator implementation from the source specification.
pub fn evaluate(
    rule: &str,
    outputs: &[Arc<Source>],
    properties: Vec<(String, String)>,
) -> Result<Box<dyn Generator>, EvaluationError> {
    let parameters = Parameters {
        outputs: outputs.into(),
        properties,
    };
    match rule {
        "gl:api" => Ok(Box::new(GLAPI::evaluate(parameters)?)),
        "gl:shaders" => Ok(Box::new(GLShaders::evaluate(parameters)?)),
        _ => Err(EvaluationError::UnknownRule(rule.to_string())),
    }
}

struct Parameters {
    outputs: Vec<Arc<Source>>,
    properties: Vec<(String, String)>,
}

impl Parameters {
    /// Get a property from the parameters, removing it.
    fn property(&mut self, name: &'static str) -> Property<String> {
        Property {
            name,
            value: self
                .properties
                .iter()
                .position(|(prop_name, _)| prop_name == name)
                .map(|index| self.properties.swap_remove(index).1),
        }
    }

    /// Finish parsing, reporting any unknown properties or inputs.
    fn done(self) -> Result<(), EvaluationError> {
        if let Some((name, _)) = self.properties.into_iter().next() {
            return Err(EvaluationError::UnknownProperty(name));
        }
        if let Some(output) = self.outputs.into_iter().next() {
            return Err(EvaluationError::UnexpectedOutput(output.path().clone()));
        }
        Ok(())
    }

    /// Get the singular output of the given type.
    fn output(&mut self, ty: SourceType) -> Result<ProjectPath, EvaluationError> {
        match self.outputs.iter().position(|src| src.ty() == ty) {
            None => Err(EvaluationError::MissingOutput(ty)),
            Some(index) => Ok(self.outputs.swap_remove(index).path().clone()),
        }
    }
}

struct Property<T> {
    name: &'static str,
    value: Option<T>,
}

impl Property<String> {
    fn parse<T: FromStr>(self) -> Result<Property<T>, EvaluationError>
    where
        <T as FromStr>::Err: 'static + error::Error,
    {
        Ok(Property {
            name: self.name,
            value: match self.value {
                None => None,
                Some(value) => match T::from_str(&value) {
                    Ok(value) => Some(value),
                    Err(err) => {
                        return Err(EvaluationError::PropertyValue(
                            self.name.to_string(),
                            err.into(),
                        ));
                    }
                },
            },
        })
    }
}

impl<T> Property<T> {
    fn required(self) -> Result<T, EvaluationError> {
        match self.value {
            None => Err(EvaluationError::MissingProperty(self.name.to_string())),
            Some(value) => Ok(value),
        }
    }
}

// ============================================================================
// GL API generator
// ============================================================================

/// OpenGL API binding generator.
#[derive(Debug)]
struct GLAPI {
    api: api::APISpec,
    link: api::APISpec,
    config: Option<Config>,
    source: ProjectPath,
    header: ProjectPath,
}

impl GLAPI {
    fn evaluate(mut params: Parameters) -> Result<Self, EvaluationError> {
        let api: api::APISpec = params.property("api").parse()?.required()?;
        let link: api::APISpec = params.property("link").parse()?.required()?;
        let config: Option<Config> = params.property("config").parse()?.value;
        let source = params.output(SourceType::Source)?;
        let header = params.output(SourceType::Header)?;
        params.done()?;
        Ok(Self {
            api,
            link,
            config,
            source,
            header,
        })
    }
}

impl Generator for GLAPI {
    fn run(
        &self,
        root: &ProjectRoot,
        sources: &SourceSpec,
    ) -> Result<Vec<Output>, Box<dyn error::Error>> {
        let api = api::API::create(&self.api, &self.link)?;
        let bindings = match &self.config {
            None => api.make_bindings(),
            Some(config) => {
                let sources = sources.sources_for_config(config)?;
                let mut flat_sources = Vec::new();
                for source in sources.sources().iter() {
                    if !source.is_generated() {
                        flat_sources.push(root.resolve(source.path()));
                    }
                }
                let entry_points = scan::read_entrypoints(&flat_sources)?;
                api.make_subset_bindings(&entry_points)?
            }
        };
        Ok(vec![
            Output {
                path: self.header.clone(),
                data: bindings.header.into(),
            },
            Output {
                path: self.source.clone(),
                data: bindings.data.into(),
            },
        ])
    }
}

// ============================================================================
// GL shader bundler
// ============================================================================

/// OpenGL shader bundler.
#[derive(Debug)]
struct GLShaders {
    source: ProjectPath,
}

impl GLShaders {
    fn evaluate(mut params: Parameters) -> Result<Self, EvaluationError> {
        let source = params.output(SourceType::Source)?;
        params.done()?;
        Ok(Self { source })
    }
}

impl Generator for GLShaders {
    fn run(
        &self,
        root: &ProjectRoot,
        _sources: &SourceSpec,
    ) -> Result<Vec<Output>, Box<dyn error::Error>> {
        let directory = root.resolve(&ProjectPath::SHADER);
        let spec_path = directory.join("shaders.txt");
        let spec = match shader::Spec::read_file(&spec_path) {
            Ok(value) => value,
            Err(err) => {
                return Err(FileError {
                    path: spec_path,
                    error: err.into(),
                }
                .into());
            }
        };
        let manifest = spec.to_manifest();
        let data = shader::Data::read_raw(&manifest, &directory)?;
        let text = data.emit_text()?;
        Ok(vec![Output {
            path: self.source.clone(),
            data: text.into(),
        }])
    }
}
