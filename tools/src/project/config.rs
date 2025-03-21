#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Variant {
    Compo,
    Full,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Linux,
    MacOS,
    Windows,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Config {
    variant: Variant,
    platform: Platform,
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
