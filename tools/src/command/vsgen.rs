use crate::project::visualstudio::Project;
use arcstr::literal;
use clap::Parser;
use std::error::Error;
use std::path::PathBuf;
use std::{env, error, fmt};
use uuid::uuid;

/// Generate Visual Studio projects.
#[derive(Parser, Debug)]
pub struct Args {
    #[arg(long)]
    output_directory: Option<PathBuf>,
}

const FILES: &[&str] = &[
    "gl_debug.cpp",
    "gl_debug.hpp",
    "gl_shader_compo.cpp",
    "gl_shader_data.cpp",
    "gl_shader_data.hpp",
    "gl_shader_full.cpp",
    "gl_shader.hpp",
    "gl_windows.cpp",
    "gl.hpp",
    "log_internal.hpp",
    "log_standard.cpp",
    "log_standard.hpp",
    "log_unix.cpp",
    "log_windows.cpp",
    "log.hpp",
    "main_windows_compo.cpp",
    "main.cpp",
    "main.hpp",
    "os_file_unix.cpp",
    "os_file_windows.cpp",
    "os_file.hpp",
    "os_string.cpp",
    "os_string.hpp",
    "os_unix.cpp",
    "os_unix.hpp",
    "os_windows.cpp",
    "os_windows.hpp",
    "scene_cube.cpp",
    "scene_cube.hpp",
    "scene_triangle.cpp",
    "scene_triangle.hpp",
    "text_buffer.cpp",
    "text_buffer.hpp",
    "text_unicode.cpp",
    "text_unicode.hpp",
    "util.hpp",
    "var_def.hpp",
    "var.cpp",
    "var.hpp",
    "wide_text_buffer.cpp",
    "wide_text_buffer.hpp",
];

#[derive(Debug, Clone, Copy)]
struct NoOutputDirectory;

impl fmt::Display for NoOutputDirectory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("no output directory specified")
    }
}

impl error::Error for NoOutputDirectory {}

impl Args {
    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        let output_directory = match &self.output_directory {
            None => {
                let Some(manifest_dir) = env::var_os("CARGO_MANIFEST_DIR") else {
                    return Err(Box::from(&NoOutputDirectory));
                };
                let mut dir = PathBuf::from(manifest_dir);
                dir.pop();
                dir
            }
            Some(value) => value.clone(),
        };

        let mut project = Project::new(uuid!("26443e89-4e15-4714-8cec-8ce4b3902761"));
        project.root_namespace = Some(literal!("demo"));
        for &file in FILES.iter() {
            if file.ends_with(".cpp") {
                &mut project.cl_compile
            } else {
                &mut project.cl_include
            }
            .push(format!("src\\{}", file).into());
        }

        project.emit(&output_directory, "LaterDarker")?;
        Ok(())
    }
}
