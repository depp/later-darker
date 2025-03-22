use crate::vsenv;
use clap::Parser;
use std::error;

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(long)]
    arch: Option<vsenv::Arch>,

    #[arg(long)]
    host_arch: Option<vsenv::Arch>,
}

impl Args {
    pub fn run(&self) -> Result<(), Box<dyn error::Error>> {
        let vs_path = vsenv::find_vs()?;
        eprintln!("Found Visual Studio: {}", vs_path);
        let mut vars = vsenv::VarCommand::new(&vs_path);
        if let Some(arch) = self.arch {
            vars.arch(arch);
        }
        if let Some(arch) = self.host_arch {
            vars.host_arch(arch);
        }
        let vars = vars.run()?;
        for (k, v) in vars.iter() {
            eprintln!("{} = {}", k, v);
        }
        Ok(())
    }
}
