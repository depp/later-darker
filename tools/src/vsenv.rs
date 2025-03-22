use std::env;
use std::error;
use std::ffi::OsString;
use std::fmt;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::str;

#[derive(Debug)]
pub enum Error {
    MissingEnvVar(&'static str),
    NoVisualStudio(io::Error),
    WhereFailed,
    WhereOutput,
    NoDirectory,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::MissingEnvVar(name) => write!(f, "missing environment variable: ${}", name),
            Error::NoVisualStudio(e) => write!(f, "could not find Visual Studio: {}", e),
            Error::WhereFailed => f.write_str("vswhere.exe could not find Visual Studio"),
            Error::WhereOutput => f.write_str("could not parse vswhere.exe output"),
            Error::NoDirectory => f.write_str("Visual Studio directory does not exist"),
        }
    }
}

impl error::Error for Error {}

fn get_env(k: &'static str) -> Result<OsString, Error> {
    env::var_os(k).ok_or(Error::MissingEnvVar(k))
}

/// Find the installation path for Visual Studio.
pub fn find_vs() -> Result<String, Error> {
    let mut vs_where = PathBuf::from(get_env("ProgramFiles(x86)")?);
    vs_where.push("Microsoft Visual Studio\\Installer\\vswhere.exe");
    let output = match Command::new(vs_where)
        .args(["-latest", "-property", "installationPath"])
        .output()
    {
        Ok(output) => output,
        Err(e) => return Err(Error::NoVisualStudio(e)),
    };
    if !output.status.success() {
        return Err(Error::WhereFailed);
    }
    let mut stdout = match String::from_utf8(output.stdout) {
        Ok(s) => s,
        Err(_) => return Err(Error::WhereOutput),
    };
    if stdout.ends_with("\r\n") {
        stdout.truncate(stdout.len() - 2);
    }
    if stdout.is_empty() {
        return Err(Error::WhereOutput);
    }
    if !Path::new(&stdout).is_dir() {
        return Err(Error::NoDirectory);
    }
    Ok(stdout)
}
