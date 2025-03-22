use std::env;
use std::error;
use std::ffi::OsString;
use std::fmt;
use std::io;
use std::os::windows::process::CommandExt as _;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::str;

#[derive(Debug)]
pub enum Error {
    MissingEnvVar(&'static str),
    ProgRun(Program, io::Error),
    ProgStatus(Program),
    ProgOutput(Program),
    NoDirectory,
}

#[derive(Debug)]
pub enum Program {
    Where,
    DevCmd,
}

impl Program {
    fn name(&self) -> &'static str {
        match self {
            Program::Where => "vswhere.exe",
            Program::DevCmd => "VsDevCmd.bat",
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::MissingEnvVar(name) => write!(f, "missing environment variable: ${}", name),
            Error::ProgRun(program, e) => {
                write!(f, "could not run program {}: {}", program.name(), e)
            }
            Error::ProgStatus(program) => write!(f, "program failed: {}", program.name()),
            Error::ProgOutput(program) => {
                write!(f, "could not output of program {}", program.name())
            }
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
    // See: https://github.com/microsoft/vswhere
    let mut vs_where = PathBuf::from(get_env("ProgramFiles(x86)")?);
    vs_where.push("Microsoft Visual Studio\\Installer\\vswhere.exe");
    let output = match Command::new(vs_where)
        .args(["-latest", "-property", "installationPath"])
        .output()
    {
        Ok(output) => output,
        Err(e) => return Err(Error::ProgRun(Program::Where, e)),
    };
    if !output.status.success() {
        return Err(Error::ProgStatus(Program::Where));
    }
    let mut stdout = match String::from_utf8(output.stdout) {
        Ok(s) => s,
        Err(_) => return Err(Error::ProgOutput(Program::Where)),
    };
    if stdout.ends_with("\r\n") {
        stdout.truncate(stdout.len() - 2);
    }
    if stdout.is_empty() {
        return Err(Error::ProgOutput(Program::Where));
    }
    if !Path::new(&stdout).is_dir() {
        return Err(Error::NoDirectory);
    }
    Ok(stdout)
}

/// Get the environment variables for a Visual Studio environment.
pub fn environment(vs_path: &str) -> Result<(), Error> {
    let cmd_exe = get_env("ComSpec")?;
    // These funny
    let output = match Command::new(cmd_exe)
        .arg("/s")
        .arg("/c")
        .raw_arg(format!(
            "\"\"{}\\Common7\\Tools\\VsDevCmd.bat\" -no_logo && set\"",
            vs_path
        ))
        .output()
    {
        Ok(output) => output,
        Err(e) => return Err(Error::ProgRun(Program::DevCmd, e)),
    };
    let stdout = match String::from_utf8(output.stdout) {
        Ok(s) => s,
        Err(_) => return Err(Error::ProgOutput(Program::DevCmd)),
    };
    for line in stdout.lines() {
        if !line.is_empty() {
            let Some((name, value)) = line.split_once('=') else {
                return Err(Error::ProgOutput(Program::DevCmd));
            };
            eprintln!("{} = {}", name, value);
        }
    }
    Ok(())
}
