use super::parse;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

/// A spec for a shader program to compile and link.
#[derive(Debug, Clone)]
pub struct Program<Shader> {
    /// Program name. Used for variable names in the generated source code.
    pub name: Arc<str>,
    /// Vertex shader source filename.
    pub vertex: Shader,
    /// Fragment shader source filename.
    pub fragment: Shader,
}

/// A spec for all shader programs to compile and link.
#[derive(Debug, Clone)]
pub struct Spec {
    pub programs: Vec<Program<Arc<str>>>,
}

impl Spec {
    /// Read a specification from a file.
    pub fn read_file(path: &Path) -> Result<Self, parse::ReadError> {
        parse::read_spec(path)
    }

    /// Convert the spec to a manifest.
    pub fn to_manifest(&self) -> Manifest {
        let mut vertex_shaders = ShaderManifest::new();
        let mut fragment_shaders = ShaderManifest::new();
        let mut programs = Vec::with_capacity(self.programs.len());
        for program in self.programs.iter() {
            programs.push(Program {
                name: program.name.clone(),
                vertex: vertex_shaders.add(&program.vertex),
                fragment: fragment_shaders.add(&program.fragment),
            });
        }
        let fragment_offset = fragment_shaders.shaders.len();
        for program in programs.iter_mut() {
            program.fragment += fragment_offset;
        }
        let mut shaders =
            Vec::with_capacity(vertex_shaders.shaders.len() + fragment_shaders.shaders.len());
        for name in vertex_shaders.shaders {
            shaders.push(Shader {
                ty: ShaderType::Vertex,
                name,
            });
        }
        for name in fragment_shaders.shaders {
            shaders.push(Shader {
                ty: ShaderType::Fragment,
                name,
            });
        }
        Manifest { shaders, programs }
    }

    pub fn dump(&self) -> String {
        use std::fmt::Write;
        let mut out = String::new();
        out.push_str("Programs:\n");
        for (n, program) in self.programs.iter().enumerate() {
            write!(
                &mut out,
                "  {}: {}; {} {}\n",
                n, program.name, program.vertex, program.fragment
            )
            .unwrap();
        }
        out
    }
}

/// A single shader to compile.
#[derive(Debug, Clone)]
pub struct Shader {
    /// The shader type.
    pub ty: ShaderType,
    /// The shader source code filename.
    pub name: Arc<str>,
}

/// A manifest for shader programs to compile and link. In a manifest, each
/// unique shader appears only once.
#[derive(Debug, Clone)]
pub struct Manifest {
    /// All shaders.
    pub shaders: Vec<Shader>,
    /// List of all programs. Shaders are indexes into the shader array above.
    pub programs: Vec<Program<usize>>,
}

impl Manifest {
    pub fn dump(&self) -> String {
        use std::fmt::Write;
        let mut out = String::new();
        out.push_str("Shaders:\n");
        for (n, shader) in self.shaders.iter().enumerate() {
            write!(&mut out, "  {}: {:?} {}\n", n, shader.ty, shader.name).unwrap();
        }
        out.push_str("Programs:\n");
        for (n, program) in self.programs.iter().enumerate() {
            write!(
                &mut out,
                "  {}: {}; {}(id={}) {}(id={})\n",
                n,
                program.name,
                self.shaders[program.vertex].name,
                program.vertex,
                self.shaders[program.fragment].name,
                program.fragment
            )
            .unwrap();
        }
        out
    }
}

/// A type of shader.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShaderType {
    Vertex,
    Fragment,
}

impl ShaderType {
    /// Get the type of a shader from its file extension.
    pub fn from_extension(ext: &str) -> Option<Self> {
        Some(match ext {
            "vert" => ShaderType::Vertex,
            "frag" => ShaderType::Fragment,
            _ => return None,
        })
    }
}

/// A manifest of shader programs of a specific type.
struct ShaderManifest {
    shaders: Vec<Arc<str>>,
    names: HashMap<Arc<str>, usize>,
}

impl ShaderManifest {
    /// Create a new shader manifest.
    fn new() -> Self {
        ShaderManifest {
            shaders: Vec::new(),
            names: HashMap::new(),
        }
    }

    /// Add a shader to the shader manifest and return its index. Returns an
    /// existing index if the shader is already present.
    fn add(&mut self, name: &Arc<str>) -> usize {
        match self.names.get(name) {
            None => {
                let index = self.shaders.len();
                self.shaders.push(name.clone());
                self.names.insert(name.clone(), index);
                index
            }
            Some(&index) => index,
        }
    }
}
