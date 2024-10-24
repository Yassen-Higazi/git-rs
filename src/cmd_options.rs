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

        #[arg(short = 's', long = "size")]
        size: bool,

        hash: String,
    },

    HashObject {
        #[arg(short = 'w', long = "write")]
        write: bool,

        #[arg(short = 't', long = "type", default_value = "blob")]
        object_type: String,

        filename: String,
    },

    LsTree {
        #[arg(long = "name-only")]
        name_only: bool,

        hash: String,
    },

    Init,

    Help,
}

impl Display for Commands {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let command_name = match self {
            Commands::Init => "init",
            Commands::Help => "help",
            Commands::LsTree { .. } => "ls-tree",
            Commands::CatFile { .. } => "cat-file",
            Commands::HashObject { .. } => "hash-object",
        };

        write!(f, "{command_name}")
    }
}
