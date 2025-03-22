use std::env;
use std::error;
use std::ffi::OsString;
use std::fmt;
use std::io;
use std::os::windows::process::CommandExt as _;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str;
use std::str::FromStr;

#[derive(Debug)]
pub enum Error {
    MissingEnvVar(&'static str),
    ProgRun(&'static str, io::Error),
    ProgStatus(&'static str),
    ProgOutput(&'static str),
    NoDirectory,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::MissingEnvVar(name) => write!(f, "missing environment variable: ${}", name),
            Error::ProgRun(program, e) => {
                write!(f, "could not run program {}: {}", program, e)
            }
            Error::ProgStatus(program) => write!(f, "program failed: {}", program),
            Error::ProgOutput(program) => {
                write!(f, "could not parse output of program {}", program)
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
    const PROGRAM: &str = "vswhere.exe";
    const PATH: &str = "Microsoft Visual Studio\\Installer\\vswhere.exe";
    let mut vs_where = PathBuf::from(get_env("ProgramFiles(x86)")?);
    vs_where.push(PATH);
    let output = match Command::new(vs_where)
        .args(["-latest", "-property", "installationPath"])
        .output()
    {
        Ok(output) => output,
        Err(e) => return Err(Error::ProgRun(PROGRAM, e)),
    };
    if !output.status.success() {
        return Err(Error::ProgStatus(PROGRAM));
    }
    let mut stdout = match String::from_utf8(output.stdout) {
        Ok(s) => s,
        Err(_) => return Err(Error::ProgOutput(PROGRAM)),
    };
    if stdout.ends_with("\r\n") {
        stdout.truncate(stdout.len() - 2);
    }
    if stdout.is_empty() {
        return Err(Error::ProgOutput(PROGRAM));
    }
    if !Path::new(&stdout).is_dir() {
        return Err(Error::NoDirectory);
    }
    Ok(stdout)
}

// Calling VsDevCmd.bat:
//   -arch=arch x86, amd64, arm, arm64
//   -host_arch=arch x86, amd64
//   -winsdk=version
//   -app_platform=platform Desktop, UPW
//   -no_ext Only run core commands
//   -no_logo No banner
//   -vcvars_ver=version
//   -vcvars_spectre_libsmode
//   -startdir=dir
//   -test
//   -help

// Note: vcvarsall.bat is a wrapper around VsDevCmd.bat, at least these days (VS
// 2022). VsDevCmd.bat is better. Commands like vcvars32.bat are just extra
// wrappers around vcvarsall.bat.

/// An architecture supported by Visual Studio tools.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arch {
    X86,
    Amd64,
    Arm,
    Arm64,
}

impl Arch {
    /// Get the architecture name.
    pub fn name(&self) -> &'static str {
        match self {
            Arch::X86 => "x86",
            Arch::Amd64 => "amd64",
            Arch::Arm => "arm",
            Arch::Arm64 => "arm64",
        }
    }
}

impl fmt::Display for Arch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// Error that indicates the architecture is unknown.
#[derive(Debug)]
pub struct UnknownArchitecture;

impl fmt::Display for UnknownArchitecture {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("unknown architecture")
    }
}

impl error::Error for UnknownArchitecture {}

impl FromStr for Arch {
    type Err = UnknownArchitecture;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "x86" | "ia32" | "win32" => Arch::X86,
            "x64" | "amd64" | "x86-64" | "x86_64" | "win64" => Arch::Amd64,
            "arm" => Arch::Arm,
            "arm64" => Arch::Arm64,
            _ => return Err(UnknownArchitecture),
        })
    }
}

/// A command to set up the Visual Studio build environment.
pub struct VarCommand {
    vs_path: PathBuf,
    arch: Option<Arch>,
    host_arch: Option<Arch>,
}

impl VarCommand {
    pub fn new(vs_path: impl AsRef<Path>) -> Self {
        VarCommand {
            vs_path: vs_path.as_ref().to_path_buf(),
            arch: None,
            host_arch: None,
        }
    }

    /// Set the target architecture.
    pub fn arch(&mut self, arch: Arch) -> &mut Self {
        self.arch = Some(arch);
        self
    }

    /// Set the host architecture.
    pub fn host_arch(&mut self, arch: Arch) -> &mut Self {
        self.host_arch = Some(arch);
        self
    }

    /// Run the command and return the environment variables.
    pub fn run(&self) -> Result<Vec<(String, String)>, Error> {
        const CMD: &str = "VsDevCmd.bat";
        let cmd_exe = get_env("ComSpec")?;
        let mut directory = self.vs_path.to_path_buf();
        directory.push("Common7\\Tools");
        let arch = self.arch.unwrap_or(Arch::X86);
        let host_arch = self.host_arch.unwrap_or(Arch::Amd64);

        // These funny quotes are necessary. With /s /c, the outermost pair of
        // quotes are stripped and the remaining command is then executed.
        let output = match Command::new(cmd_exe)
            .current_dir(directory)
            .arg("/s")
            .arg("/c")
            .raw_arg(format!(
                "\"{} -no_logo -arch={} -host_arch={} && set\"",
                CMD, arch, host_arch
            ))
            .output()
        {
            Ok(output) => output,
            Err(e) => return Err(Error::ProgRun(CMD, e)),
        };
        eprintln!("Output: {}", String::from_utf8_lossy(&output.stdout));
        let stdout = match String::from_utf8(output.stdout) {
            Ok(s) => s,
            Err(_) => return Err(Error::ProgOutput(CMD)),
        };
        let mut result = Vec::new();
        for line in stdout.lines() {
            if !line.is_empty() {
                let Some((name, value)) = line.split_once('=') else {
                    return Err(Error::ProgOutput(CMD));
                };
                result.push((name.to_string(), value.to_string()));
            }
        }
        Ok(result)
    }
}
