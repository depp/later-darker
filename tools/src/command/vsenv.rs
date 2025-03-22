use crate::vsenv;
use clap::Parser;
use std::error;

#[derive(Parser, Debug)]
pub struct Args;

impl Args {
    pub fn run(&self) -> Result<(), Box<dyn error::Error>> {
        let path = vsenv::find_vs()?;
        eprintln!("Found Visual Studio: {}", path);
        Ok(())
    }
}
