use clap::Parser;
use git::Git;

use crate::cmd_options::CmdOptions;

mod cmd_options;
mod git;

fn main() -> anyhow::Result<()> {
    let options = CmdOptions::parse();

    println!("Options: {:?}", options);

    let git = Git::new();

    git.execute(&options.command)?;

    Ok(())
}
