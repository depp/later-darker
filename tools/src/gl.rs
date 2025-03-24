use crate::emit;
use crate::xmlparse::{
    self, element_children_tag, element_children_unchecked, node_pos, require_attribute,
};
use arcstr::ArcStr;
use khronos_api;
use roxmltree::{self, Document, Node, NodeType, TextPos};
use std::collections::{HashMap, HashSet};
use std::error;
use std::fmt::{self, Write as _};
use std::str;

const LINKABLE_VERSION: Version = Version(1, 1);
const MAX_VERSION: Version = Version(3, 3);

// ============================================================================
// API spec
// ============================================================================

/// OpenGL API version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version(u8, u8);

impl Version {
    fn parse(text: &str) -> Option<Self> {
        let (major, minor) = text.split_once('.')?;
        let major = u8::from_str_radix(major, 10).ok()?;
        let minor = u8::from_str_radix(minor, 10).ok()?;
        Some(Version(major, minor))
    }
}

/// Error parsing an OpenGL API specification.
#[derive(Debug)]
pub enum APISpecParseError {
    InvalidVersion,
    Empty,
}

impl fmt::Display for APISpecParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use APISpecParseError::*;
        match self {
            InvalidVersion => f.write_str("invalid version"),
            Empty => f.write_str("empty spec"),
        }
    }
}

impl error::Error for APISpecParseError {}

/// Specification for a subset of the OpenGL API. Specifies which version and
/// extensions are included.
#[derive(Debug, Clone)]
pub struct APISpec {
    pub version: Version,
    pub extensions: Vec<String>,
}

impl str::FromStr for APISpec {
    type Err = APISpecParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split_ascii_whitespace();
        let version = parts.next().ok_or(APISpecParseError::Empty)?;
        let version = Version::parse(version).ok_or(APISpecParseError::InvalidVersion)?;
        let mut extensions = Vec::new();
        for part in parts {
            extensions.push(part.to_string());
        }
        Ok(Self {
            version,
            extensions,
        })
    }
}

// ============================================================================
// Error
// ============================================================================

/// An error genrating the OpenGL API.
#[derive(Debug)]
pub enum Error {
    MissingCommandProto(TextPos),
    MissingCommandName(TextPos),
    InvalidVersion(String, TextPos),
    InvalidRemoveProfile(TextPos),
    DuplicateEnum(String),
    InvalidPrototype(TextPos),
    AliasConflict(String, String),
    UnknownType(String, TextPos),
    Parse(roxmltree::Error),
    XML(xmlparse::Error),
}

impl From<roxmltree::Error> for Error {
    fn from(value: roxmltree::Error) -> Self {
        Error::Parse(value)
    }
}

impl From<xmlparse::Error> for Error {
    fn from(value: xmlparse::Error) -> Self {
        Error::XML(value)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Error::*;
        match self {
            MissingCommandProto(pos) => write!(f, "missing command <proto> at {}", pos),
            MissingCommandName(pos) => write!(f, "missing command <name> at {}", pos),
            InvalidVersion(version, pos) => {
                write!(f, "invalid version number {:?} at {}", version, pos)
            }
            InvalidRemoveProfile(pos) => write!(f, "invalid profile for remove at {}", pos),
            DuplicateEnum(name) => write!(f, "duplicate enum {:?}", name),
            InvalidPrototype(pos) => write!(f, "invalid prototype at {}", pos),
            AliasConflict(name, alias) => write!(
                f,
                "enum {:?} is alias for {:?}, but that has a conflicting definiton",
                name, alias
            ),
            UnknownType(name, pos) => write!(f, "unknown type {:?} at {}", name, pos),
            Parse(err) => err.fmt(f),
            XML(err) => err.fmt(f),
        }
    }
}

impl error::Error for Error {}

// ============================================================================
// Feature & Version Map
// ============================================================================

/// How OpenGL functions are called.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CallType {
    /// The call can resolved at link-time.
    Linker,
    /// The function must be loaded by pointer at runtime.
    Runtime,
}

/// A set of features included in an API.
struct FeatureSet<'a> {
    enums: HashSet<&'a str>,
    commands: HashMap<&'a str, CallType>,
}

impl<'a> FeatureSet<'a> {
    fn build(node: Node<'a, 'a>) -> Result<Self, Error> {
        assert_eq!(node.tag_name().name(), "registry");
        let mut set: FeatureSet<'_> = FeatureSet {
            enums: HashSet::new(),
            commands: HashMap::new(),
        };
        for child in node.children() {
            if child.is_element() && child.tag_name().name() == "feature" {
                set.parse_feature(child)?;
            }
        }
        Ok(set)
    }

    fn parse_feature(&mut self, node: Node<'a, 'a>) -> Result<(), Error> {
        assert_eq!(node.tag_name().name(), "feature");
        if require_attribute(node, "api")? != "gl" {
            return Ok(());
        }
        let version = require_attribute(node, "number")?;
        let version = match Version::parse(version) {
            None => {
                return Err(Error::InvalidVersion(version.into(), node_pos(node)));
            }
            Some(version) => version,
        };
        let availability = if version <= LINKABLE_VERSION {
            CallType::Linker
        } else if version <= MAX_VERSION {
            CallType::Runtime
        } else {
            return Ok(());
        };
        for child in node.children() {
            if child.is_element() {
                match child.tag_name().name() {
                    "require" => self.parse_require(child, availability)?,
                    "remove" => self.parse_remove(child)?,
                    _ => return Err(xmlparse::unexpected_tag(child).into()),
                }
            }
        }
        Ok(())
    }

    fn parse_require(&mut self, node: Node<'a, 'a>, availability: CallType) -> Result<(), Error> {
        assert_eq!(node.tag_name().name(), "require");
        for child in node.children() {
            if child.is_element() {
                match child.tag_name().name() {
                    "command" => {
                        let name = require_attribute(child, "name")?;
                        self.commands.insert(name, availability);
                    }
                    "enum" => {
                        let name = require_attribute(child, "name")?;
                        self.enums.insert(name);
                    }
                    "type" => (),
                    _ => return Err(xmlparse::unexpected_tag(child).into()),
                }
            }
        }
        Ok(())
    }

    fn parse_remove(&mut self, node: Node<'a, 'a>) -> Result<(), Error> {
        assert_eq!(node.tag_name().name(), "remove");
        let profile = require_attribute(node, "profile")?;
        if profile != "core" {
            return Err(Error::InvalidRemoveProfile(node_pos(node)));
        }
        for child in node.children() {
            if child.is_element() {
                match child.tag_name().name() {
                    "command" => {
                        let name = require_attribute(child, "name")?;
                        self.commands.remove(name);
                    }
                    "enum" => {
                        let name = require_attribute(child, "name")?;
                        self.enums.remove(name);
                    }
                    "type" => (),
                    _ => return Err(xmlparse::unexpected_tag(child).into()),
                }
            }
        }
        Ok(())
    }
}

// ============================================================================
// Enums
// ============================================================================

/// Emit enum value definitions.
fn emit_enums<'a>(
    enums: &HashSet<&str>,
    node: Node<'a, 'a>,
    type_map: &TypeMap,
) -> Result<String, Error> {
    let mut out = String::new();
    let mut emitted: HashMap<&str, (&str, &str)> = HashMap::with_capacity(enums.len());
    for child in element_children_tag(node, "enums") {
        let ty = match child.attribute("type") {
            None => "GLenum",
            Some(s) => match s {
                "bitmask" => "GLbitfield",
                _ => panic!("type {:?}", s),
            },
        };
        let ty = type_map.map(ty);
        for item in element_children_unchecked(child) {
            match item.tag_name().name() {
                "enum" => {
                    if let Some(api) = item.attribute("api") {
                        if api != "gl" {
                            continue;
                        }
                    }
                    let name = require_attribute(item, "name")?;
                    if !enums.contains(name) {
                        continue;
                    }
                    if emitted.contains_key(name) {
                        return Err(Error::DuplicateEnum(name.into()));
                    }
                    let ty = match item.attribute("type") {
                        None => ty,
                        Some(t) => match t {
                            "u" => "unsigned",
                            "ull" => "unsigned long long",
                            _ => return Err(Error::UnknownType(t.into(), node_pos(item))),
                        },
                    };
                    let value = require_attribute(item, "value")?;
                    let definition = (ty, value);
                    let value = match item.attribute("alias") {
                        None => value,
                        Some(alias) => match emitted.get(alias) {
                            None => value,
                            Some(&alias_definition) => {
                                if definition != alias_definition {
                                    return Err(Error::AliasConflict(name.into(), alias.into()));
                                }
                                alias
                            }
                        },
                    };
                    writeln!(out, "constexpr {} {} = {};", ty, name, value).unwrap();
                    emitted.insert(name, definition);
                }
                "unused" => (),
                _ => return Err(xmlparse::unexpected_tag(item).into()),
            }
        }
    }
    Ok(out)
}

// ============================================================================
// Functions
// ============================================================================

#[derive(Debug, Clone)]
struct Function {
    name: ArcStr,
    call: CallType,
    return_type: String,
    parameter_declarations: String,
    parameter_names: String,
}

/// Get the name and prototype for a command.
fn command_info<'a>(node: Node<'a, 'a>) -> Result<(String, Node<'a, 'a>), Error> {
    assert_eq!(node.tag_name().name(), "command");
    let Some(proto) = element_children_tag(node, "proto").next() else {
        return Err(Error::MissingCommandProto(node_pos(node)));
    };
    let Some(name) = element_children_tag(proto, "name").next() else {
        return Err(Error::MissingCommandName(node_pos(proto)));
    };
    Ok((xmlparse::parse_text_contents(name)?, proto))
}

/// Emit the return type of a function, given the <proto> tag.
fn emit_return_type<'a>(node: Node<'a, 'a>, type_map: &TypeMap) -> Result<String, Error> {
    let mut out = String::new();
    let mut has_name = false;
    for child in node.children() {
        match child.node_type() {
            NodeType::Element => match child.tag_name().name() {
                "name" => has_name = true,
                "ptype" => {
                    if has_name {
                        return Err(Error::InvalidPrototype(node_pos(node)));
                    }
                    let ty = xmlparse::parse_text_contents(child)?;
                    out.push_str(type_map.map(&ty));
                }
                _ => return Err(xmlparse::unexpected_tag(child).into()),
            },
            NodeType::Text => {
                if let Some(text) = child.text() {
                    if !has_name {
                        out.push_str(text);
                    } else if text.chars().any(|c| !c.is_ascii_whitespace()) {
                        return Err(Error::InvalidPrototype(node_pos(node)));
                    }
                }
            }
            _ => (),
        }
    }
    let len = out.trim_ascii_end().len();
    if len == 0 {
        return Err(Error::InvalidPrototype(node_pos(node)));
    }
    out.truncate(len);
    Ok(out)
}

/// Emit the parameter declarations and parameter names, given the <command>
/// tag.
fn emit_parameters<'a>(node: Node<'a, 'a>, type_map: &TypeMap) -> Result<(String, String), Error> {
    let mut declarations = String::new();
    let mut names = String::new();
    let mut has_parameter = false;
    for child in element_children_tag(node, "param") {
        if has_parameter {
            declarations.push_str(", ");
            names.push_str(", ");
        }
        has_parameter = true;
        let mut has_name = false;
        for item in child.children() {
            match item.node_type() {
                NodeType::Element => match item.tag_name().name() {
                    "ptype" => {
                        let ty = xmlparse::parse_text_contents(item)?;
                        declarations.push_str(type_map.map(&ty));
                    }
                    "name" => {
                        if has_name {
                            return Err(Error::InvalidPrototype(node_pos(item)));
                        }
                        has_name = true;
                        let pos = declarations.len();
                        xmlparse::append_text_contents(&mut declarations, item)?;
                        names.push_str(&declarations[pos..]);
                    }
                    _ => return Err(xmlparse::unexpected_tag(item).into()),
                },
                NodeType::Text => {
                    if let Some(text) = item.text() {
                        declarations.push_str(text);
                    }
                }
                _ => (),
            }
        }
        if !has_name {
            return Err(Error::InvalidPrototype(node_pos(child)));
        }
    }
    Ok((declarations, names))
}

impl Function {
    /// Parse an individual command, if it is in the command list. Otherwise
    /// return None.
    fn parse(
        commands: &HashMap<&str, CallType>,
        node: Node,
        type_map: &TypeMap,
    ) -> Result<Option<Self>, Error> {
        let (name, proto) = command_info(node)?;
        let Some(&call) = commands.get(name.as_str()) else {
            return Ok(None);
        };
        let return_type = emit_return_type(proto, type_map)?;
        let (parameter_declarations, parameter_names) = emit_parameters(node, type_map)?;
        Ok(Some(Function {
            name: name.into(),
            call,
            return_type,
            parameter_declarations,
            parameter_names,
        }))
    }

    /// Parse all commands in the command list.
    fn parse_all(
        commands: &HashMap<&str, CallType>,
        node: Node,
        type_map: &TypeMap,
    ) -> Result<Vec<Self>, Error> {
        let mut result = Vec::with_capacity(commands.len());
        for child in element_children_tag(node, "commands") {
            for item in element_children_unchecked(child) {
                if item.tag_name().name() != "command" {
                    return Err(xmlparse::unexpected_tag(item).into());
                }
                if let Some(function) = Self::parse(commands, item, type_map)? {
                    result.push(function);
                }
            }
        }
        Ok(result)
    }

    /// Emit a linked API binding to this function.
    fn emit_linked(&self, out: &mut String) {
        write!(
            out,
            "GLIMPORT {} GLAPI {}({});\n",
            self.return_type, self.name, self.parameter_declarations
        )
        .unwrap();
    }

    /// Emit a missing binding to this function, which may not be called.
    fn emit_missing(&self, out: &mut String) {
        writeln!(
            out,
            "{} {}({}); // undefined",
            self.return_type, self.name, self.parameter_declarations
        )
        .unwrap();
    }

    /// Emit a runtime binding to this function.
    fn emit_runtime(&self, out: &mut String, index: usize) {
        write!(
            out,
            "inline {} {}({}) {{\n\
            \tusing Proc = {} (GLAPI *)({});\n\t",
            self.return_type,
            self.name,
            self.parameter_declarations,
            self.return_type,
            self.parameter_declarations
        )
        .unwrap();
        if self.return_type != "void" {
            out.push_str("return ");
        }
        write!(
            out,
            "static_cast<Proc>(demo::gl_api::FunctionPointers[{}])({});\n}}\n",
            index, self.parameter_names
        )
        .unwrap();
    }
}

// ============================================================================
// API
// ============================================================================

/// An OpenGL API subset.
pub struct API {
    enums: String,
    functions: Vec<Function>,
}

impl API {
    fn parse(node: Node) -> Result<Self, Error> {
        let type_map = TypeMap::create();
        if node.tag_name().name() != "registry" {
            return Err(xmlparse::unexpected_tag(node).into());
        }
        let features = FeatureSet::build(node)?;
        let enums = emit_enums(&features.enums, node, &type_map)?;
        let functions = Function::parse_all(&features.commands, node, &type_map)?;
        Ok(API { enums, functions })
    }

    /// Create an OpenGL API.
    pub fn create() -> Result<Self, Error> {
        let spec_data = khronos_api::GL_XML;
        let spec_data = str::from_utf8(spec_data).expect("XML registry is not UTF-8.");
        let doc = Document::parse(spec_data)?;
        Self::parse(doc.root_element())
    }

    /// Create bindings for this API.
    pub fn make_bindings(&self) -> Bindings {
        self.make_bindings_impl(None)
    }

    /// Create bindings for a subset of this API.
    pub fn make_subset_bindings<T>(&self, subset: T) -> Result<Bindings, UnknownFunctions>
    where
        T: IntoIterator,
        <<T as IntoIterator>::IntoIter as Iterator>::Item: AsRef<str>,
    {
        let mut all: HashSet<&str> = HashSet::new();
        for function in self.functions.iter() {
            all.insert(function.name.as_str());
        }
        let mut set: HashSet<&str> = HashSet::new();
        let mut unknown = Vec::new();
        for item in subset.into_iter() {
            let item = item.as_ref();
            match all.get(item) {
                None => unknown.push(item.to_string()),
                Some(&s) => {
                    set.insert(s);
                }
            }
        }
        if !unknown.is_empty() {
            return Err(UnknownFunctions(unknown));
        }
        Ok(self.make_bindings_impl(Some(&set)))
    }

    fn make_bindings_impl(&self, subset: Option<&HashSet<&str>>) -> Bindings {
        let functions = Functions::emit(self, subset);
        Bindings {
            header: emit_header(&self.enums, &functions),
            data: emit_data(&functions),
        }
    }
}

/// Indicates that some requested functions do not exist in this API.
#[derive(Debug)]
pub struct UnknownFunctions(Vec<String>);

impl fmt::Display for UnknownFunctions {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("unknown OpenGL functions: ")?;
        for (n, function) in self.0.iter().enumerate() {
            if n != 0 {
                f.write_str(", ")?;
            }
            if !function.is_empty()
                && function
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_')
            {
                f.write_str(function)?;
            } else {
                write!(f, "{:?}", function)?;
            }
        }
        Ok(())
    }
}

impl error::Error for UnknownFunctions {}

// ============================================================================
// Bindings
// ============================================================================

struct Functions {
    functions: String,
    lookups: Vec<ArcStr>,
}

impl Functions {
    fn emit(api: &API, subset: Option<&HashSet<&str>>) -> Self {
        let mut functions = String::new();
        let mut lookups: Vec<ArcStr> = Vec::new();
        for function in api.functions.iter() {
            match function.call {
                CallType::Linker => function.emit_linked(&mut functions),
                CallType::Runtime => {
                    let include = match subset {
                        None => true,
                        Some(set) => set.contains(function.name.as_str()),
                    };
                    if include {
                        let index = lookups.len();
                        lookups.push(function.name.clone());
                        function.emit_runtime(&mut functions, index);
                    } else {
                        function.emit_missing(&mut functions);
                    }
                }
            }
        }
        Functions { functions, lookups }
    }
}

/// Generated OpenGL API bindings.
pub struct Bindings {
    pub header: String,
    pub data: String,
}

fn emit_header(enums: &str, functions: &Functions) -> String {
    let mut out = String::new();
    out.push_str(emit::HEADER);
    out.push_str(
        "#define GLAPI __stdcall\n\
        #define GLIMPORT __declspec(dllimport)\n\
        namespace demo {\n\
        namespace gl_api {\n",
    );
    writeln!(
        out,
        "constexpr int FunctionPointerCount = {};",
        functions.lookups.len()
    )
    .unwrap();
    out.push_str(
        "extern void *FunctionPointers[FunctionPointerCount];\n\
        extern const char FunctionNames[];\n\
        }\n\
        }\n\
        \n\
        // Constants \n\
        \n",
    );
    out.push_str(&enums);
    out.push_str(
        "\n\
        // Functions\n\
        \n\
        extern \"C\" {\n\
        ",
    );
    out.push_str(&functions.functions);
    out.push_str("}\n");
    out
}

fn emit_data(functions: &Functions) -> String {
    let mut out = String::new();
    out.push_str(emit::HEADER);

    out.push_str(
        "namespace demo {\n\
        namespace gl_api {\n",
    );
    let size = functions
        .lookups
        .iter()
        .map(|name| name.len())
        .sum::<usize>()
        + functions.lookups.len();
    writeln!(
        out,
        "void *FunctionPointers[{}];\n\
        extern const char FunctionNames[{}] =",
        functions.lookups.len(),
        size
    )
    .unwrap();
    let mut writer = emit::StringWriter::new(&mut out);
    for (n, name) in functions.lookups.iter().enumerate() {
        if n != 0 {
            writer.write(&[0]);
        }
        writer.write(name.as_bytes());
    }
    writer.finish();
    out.push_str(";\n}\n}\n");
    out
}

struct TypeMap(HashMap<&'static str, &'static str>);

impl TypeMap {
    fn create() -> Self {
        TypeMap(HashMap::from_iter(TYPE_MAP.iter().cloned()))
    }

    fn map<'a>(&'_ self, ty: &'a str) -> &'a str {
        self.0.get(ty).cloned().unwrap_or(ty)
    }
}

const TYPE_MAP: &[(&str, &str)] = &[
    // ("GLenum", "unsigned"),
    ("GLboolean", "unsigned char"),
    ("GLbitfield", "unsigned"),
    ("GLbyte", "char"),
    ("GLubyte", "unsigned char"),
    ("GLshort", "short"),
    ("GLushort", "unsigned short"),
    ("GLint", "int"),
    ("GLuint", "unsigned"),
    ("GLsizei", "int"),
    ("GLfloat", "float"),
    ("GLclampf", "float"),
    ("GLdouble", "double"),
    ("GLclampd", "double"),
    ("GLchar", "char"),
    // GLhalf
    // GLfixed
    ("GLintptr", "long long"),
    ("GLsizeiptr", "long long"),
    ("GLint64", "long long"),
    ("GLuint64", "unsigned long long"),
];
