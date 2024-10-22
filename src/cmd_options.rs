use clap::{Parser, Subcommand};
use std::fmt::Display;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct CmdOptions {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    CatFile {
        #[arg(short = 't', long = "type")]
        print_file_type: bool,

        #[arg(short = 'p', long = "pretty_print")]
        pretty_print: bool,

        #[arg(short = 'c', long = "size")]
        size: bool,

        hash: String,
    },

    Init,

    Help,
}

impl Display for Commands {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[allow(unused_assignments)]
        let mut command_name: &str = "";

        match self {
            Commands::CatFile { .. } => command_name = "cat-file",
            Commands::Init => command_name = "init",
            Commands::Help => command_name = "help",
        }

        write!(f, "{command_name}")
    }
}
