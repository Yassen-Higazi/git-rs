use anyhow::Context;

use crate::cmd_options::Commands;
use std::fs;

pub struct Git {}

impl Git {
    pub fn new() -> Self {
        Self {}
    }

    pub fn execute(&self, command: &Commands) -> anyhow::Result<()> {
        match command {
            Commands::Init => {
                self.create_directory(".git")?;
                self.create_directory(".git/refs")?;
                self.create_directory(".git/objects")?;

                fs::write(".git/HEAD", "ref: refs/heads/main\n")?;

                println!("Initialized git directory")
            }

            _ => println!("Unsupported command: {}", command),
        }

        Ok(())
    }

    fn create_directory(&self, dir_name: &str) -> anyhow::Result<()> {
        fs::create_dir(dir_name).with_context(|| format!("Could not create directory {dir_name}"))
    }
}
