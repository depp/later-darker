use super::paths::ProjectRoot;
use serde::Serialize;
use std::env;
use std::error;
use std::fmt;
use std::io;
use std::path::Path;
use std::process::{Command, Stdio};

fn is_false(value: &bool) -> bool {
    !value
}

/// Information about the build.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildInfo {
    /// The time the build was initiated.
    pub build_time: chrono::DateTime<chrono::Utc>,

    /// Git commit used for the build.
    pub commit: String,

    #[serde(skip_serializing_if = "is_false")]
    pub is_dirty: bool,
}

impl BuildInfo {
    /// Get the build info for the project.
    pub fn query(project_root: &ProjectRoot) -> Result<Self, BuildInfoError> {
        let build_time = chrono::Utc::now();
        let commit = get_commit(project_root)?;
        let is_dirty = git_is_dirty(project_root.as_path())?;
        Ok(Self {
            commit,
            is_dirty,
            build_time,
        })
    }
}

#[derive(Debug)]
pub enum BuildInfoError {
    GitParse(Box<dyn error::Error>),
    GitStatus,
    GitRun(io::Error),
    VarError(&'static str, env::VarError),
}

impl fmt::Display for BuildInfoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BuildInfoError::GitParse(err) => write!(f, "could not parse git output: {}", err),
            BuildInfoError::GitStatus => f.write_str("git returned error status"),
            BuildInfoError::GitRun(err) => write!(f, "could not run git: {}", err),
            BuildInfoError::VarError(key, err) => write!(f, "could not parse ${}: {}", key, err),
        }
    }
}

impl error::Error for BuildInfoError {}

/// Get an environment variable value.
fn get_var(key: &'static str) -> Result<Option<String>, BuildInfoError> {
    match env::var(key) {
        Ok(value) => Ok(Some(value)),
        Err(env::VarError::NotPresent) => Ok(None),
        Err(err) => Err(BuildInfoError::VarError(key, err)),
    }
}

/// Get the Git commit for a specific directory.
fn git_get_commit(path: &Path) -> Result<String, BuildInfoError> {
    eprintln!("Getting commit");
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(path)
        .stderr(Stdio::inherit())
        .output()
        .map_err(BuildInfoError::GitRun)?;
    if !output.status.success() {
        return Err(BuildInfoError::GitStatus);
    }
    let mut value =
        String::from_utf8(output.stdout).map_err(|err| BuildInfoError::GitParse(err.into()))?;
    value.truncate(value.trim_ascii_end().len());
    eprintln!("Commit is {:?}", value);
    Ok(value)
}

/// Get the Git commit for the main project.
fn get_commit(project_root: &ProjectRoot) -> Result<String, BuildInfoError> {
    const GITHUB_SHA: &str = "GITHUB_SHA";
    if let Some(value) = get_var(GITHUB_SHA)? {
        eprintln!("Commit is {}={:?}", GITHUB_SHA, value);
        return Ok(value);
    }
    git_get_commit(project_root.as_path())
}

/// Test if the repository is dirty.
fn git_is_dirty(path: &Path) -> Result<bool, BuildInfoError> {
    eprintln!("Getting Git status");
    let output = Command::new("git")
        .args(["diff-index", "--quiet", "HEAD", "--"])
        .current_dir(path)
        .status()
        .map_err(BuildInfoError::GitRun)?;
    Ok(!output.success())
}
