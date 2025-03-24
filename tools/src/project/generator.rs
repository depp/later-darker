use super::paths::{ProjectPath, ProjectRoot};
use super::sources::Source;
use crate::error::FileError;
use crate::gl;
use crate::project::sources::SourceType;
use crate::shader;
use std::error;
use std::fmt;
use std::sync::Arc;

// ============================================================================
// Errors
// ============================================================================

/// An error constructing a generator rule.
#[derive(Debug)]
pub enum EvaluationError {
    UnknownRule(String),
    BadOutputs,
}

impl fmt::Display for EvaluationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EvaluationError::UnknownRule(name) => write!(f, "unknown generator rule: {:?}", name),
            EvaluationError::BadOutputs => write!(f, "invalid outputs for this rule"),
        }
    }
}

impl error::Error for EvaluationError {}

// ============================================================================
// Errors
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
) -> Result<Box<dyn Generator>, EvaluationError> {
    match rule {
        "gl:api" => Ok(Box::new(GLAPI::evaluate(outputs)?)),
        "gl:shaders" => Ok(Box::new(GLShaders::evaluate(outputs)?)),
        _ => Err(EvaluationError::UnknownRule(rule.to_string())),
    }
}

// ============================================================================
// GL API generator
// ============================================================================

/// OpenGL API binding generator.
#[derive(Debug)]
struct GLAPI {
    source: ProjectPath,
    header: ProjectPath,
}

impl GLAPI {
    fn evaluate(outputs: &[Arc<Source>]) -> Result<Self, EvaluationError> {
        let mut source = None;
        let mut header = None;
        for output in outputs.iter() {
            match output.ty() {
                SourceType::Source => {
                    if source.is_some() {
                        return Err(EvaluationError::BadOutputs);
                    }
                    source = Some(output.path().clone());
                }
                SourceType::Header => {
                    if header.is_some() {
                        return Err(EvaluationError::BadOutputs);
                    }
                    header = Some(output.path().clone());
                }
            }
        }
        let source = source.ok_or(EvaluationError::BadOutputs)?;
        let header = header.ok_or(EvaluationError::BadOutputs)?;
        Ok(Self { source, header })
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
    fn evaluate(outputs: &[Arc<Source>]) -> Result<Self, EvaluationError> {
        let mut source = None;
        for output in outputs.iter() {
            match output.ty() {
                SourceType::Source => {
                    if source.is_some() {
                        return Err(EvaluationError::BadOutputs);
                    }
                    source = Some(output.path().clone());
                }
                _ => return Err(EvaluationError::BadOutputs),
            }
        }
        let source = source.ok_or(EvaluationError::BadOutputs)?;
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
