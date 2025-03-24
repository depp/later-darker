use super::paths::{ProjectPath, ProjectRoot};
use super::sources::Source;
use crate::error::FileError;
use crate::gl;
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
    fn run(&self, root: &ProjectRoot) -> Result<Vec<Output>, Box<dyn error::Error>>;
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
    api: gl::APISpec,
    link: gl::APISpec,
    source: ProjectPath,
    header: ProjectPath,
}

impl GLAPI {
    fn evaluate(mut params: Parameters) -> Result<Self, EvaluationError> {
        let api: gl::APISpec = params.property("api").parse()?.required()?;
        let link: gl::APISpec = params.property("link").parse()?.required()?;
        let source = params.output(SourceType::Source)?;
        let header = params.output(SourceType::Header)?;
        params.done()?;
        Ok(Self {
            api,
            link,
            source,
            header,
        })
    }
}

impl Generator for GLAPI {
    fn run(&self, _: &ProjectRoot) -> Result<Vec<Output>, Box<dyn error::Error>> {
        let api = gl::API::create()?.make_bindings();
        Ok(vec![
            Output {
                path: self.header.clone(),
                data: api.header.into(),
            },
            Output {
                path: self.source.clone(),
                data: api.data.into(),
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
    fn run(&self, root: &ProjectRoot) -> Result<Vec<Output>, Box<dyn error::Error>> {
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
