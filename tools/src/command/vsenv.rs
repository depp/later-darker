use crate::vsenv;
use clap::Parser;
use std::error;

#[derive(Parser, Debug)]
pub struct Args;

impl Args {
    pub fn run(&self) -> Result<(), Box<dyn error::Error>> {
        let vs_path = vsenv::find_vs()?;
        eprintln!("Found Visual Studio: {}", vs_path);
        vsenv::environment(&vs_path)?;
        Ok(())
    }
}
