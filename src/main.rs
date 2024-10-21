#[allow(unused_imports)]
use std::env;
#[allow(unused_imports)]
use std::fs;

use anyhow::Context;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args[1] == "init" {
        create_directory(".git")?;
        create_directory(".git/objects")?;
        create_directory(".git/refs")?;

        fs::write(".git/HEAD", "ref: refs/heads/main\n")?;

        println!("Initialized git directory")
    } else {
        println!("unknown command: {}", args[1])
    }

    Ok(())
}

fn create_directory(dir_name: &str) -> anyhow::Result<()> {
    fs::create_dir(dir_name).with_context(|| format!("Could not create directory {dir_name}"))
}
