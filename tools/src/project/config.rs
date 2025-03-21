use std::str::FromStr;
use std::{error, fmt};

#[derive(Debug, Clone, Copy)]
pub struct UnknownVariant;

impl fmt::Display for UnknownVariant {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("unknown config variant")
    }
}

impl error::Error for UnknownVariant {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Variant {
    Compo,
    Full,
}

impl FromStr for Variant {
    type Err = UnknownVariant;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(if s.eq_ignore_ascii_case("compo") {
            Variant::Compo
        } else if s.eq_ignore_ascii_case("full") {
            Variant::Full
        } else {
            return Err(UnknownVariant);
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct UnknownPlatform;

impl fmt::Display for UnknownPlatform {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("unknown platform")
    }
}

impl error::Error for UnknownPlatform {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Linux,
    MacOS,
    Windows,
}

impl FromStr for Platform {
    type Err = UnknownPlatform;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(if s.eq_ignore_ascii_case("linux") {
            Platform::Linux
        } else if s.eq_ignore_ascii_case("macos") {
            Platform::MacOS
        } else if s.eq_ignore_ascii_case("windows") {
            Platform::Windows
        } else {
            return Err(UnknownPlatform);
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Config {
    platform: Platform,
    variant: Variant,
}

impl Config {
    /// Evaluate a build tag in this config.
    pub fn eval_tag(&self, tag: &str) -> Option<bool> {
        Some(match tag {
            "compo" => self.variant == Variant::Compo,
            "full" => self.variant == Variant::Full,
            "windows" => self.platform == Platform::Windows,
            "unix" => self.platform != Platform::Windows,
            "macos" => self.platform == Platform::MacOS,
            "linux" => self.platform == Platform::Linux,
            _ => return None,
        })
    }
}

/// Test if the string is a recognized as a build tag.
pub fn is_tag(tag: &str) -> bool {
    match tag {
        "compo" | "full" | "windows" | "unix" | "macos" | "linux" => true,
        _ => false,
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ParseConfigError {
    InvalidSyntax,
    Variant(UnknownVariant),
    Platform(UnknownPlatform),
}

impl fmt::Display for ParseConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvalidSyntax => f.write_str("invalid syntax"),
            Self::Variant(e) => e.fmt(f),
            Self::Platform(e) => e.fmt(f),
        }
    }
}

impl error::Error for ParseConfigError {}

impl FromStr for Config {
    type Err = ParseConfigError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some((platform, variant)) = s.split_once(':') else {
            return Err(ParseConfigError::InvalidSyntax);
        };
        let platform = match Platform::from_str(platform) {
            Ok(value) => value,
            Err(e) => return Err(ParseConfigError::Platform(e)),
        };
        let variant = match Variant::from_str(variant) {
            Ok(value) => value,
            Err(e) => return Err(ParseConfigError::Variant(e)),
        };
        Ok(Config { platform, variant })
    }
}
