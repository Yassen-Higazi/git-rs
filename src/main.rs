use clap::Parser;

use cmd_options::CmdOptions;
use git::Git;

mod cmd_options;
mod git;
mod git_objects;
mod utils;

fn main() -> anyhow::Result<()> {
    let options = CmdOptions::parse();

    let git = Git::new();

    git.execute(&options.command)?;

    Ok(())
}
