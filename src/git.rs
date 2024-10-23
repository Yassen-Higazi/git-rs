use anyhow::bail;

use crate::utils::*;
use crate::{cmd_options::Commands, git_objects::GitObject};

pub struct Git {}

impl Git {
    pub fn new() -> Self {
        Self {}
    }

    pub fn execute(&self, command: &Commands) -> anyhow::Result<()> {
        match command {
            Commands::Init => {
                create_directory(".git")?;
                create_directory(".git/refs")?;
                create_directory(".git/objects")?;

                write_to_file(".git/HEAD", b"ref: refs/heads/main\n")?;

                println!("Initialized git directory")
            }

            Commands::CatFile {
                print_file_type,
                pretty_print,
                hash,
                size,
            } => {
                let compressed_content = read_object(hash.as_str())?;

                let object = GitObject::from_file_content(hash.to_owned(), compressed_content)?;

                if *pretty_print {
                    object.print_content();
                } else if *print_file_type {
                    object.print_type();
                } else if *size {
                    object.print_size()?;
                } else {
                    bail!("Invalid command");
                }
            }

            Commands::HashObject {
                write,
                object_type,
                filename,
            } => {
                let content = read_file(filename)?;

                let object = GitObject::from_file_content_and_type(
                    object_type,
                    String::from_utf8(content)?,
                    None,
                )?;

                if *write {
                    object.write_to_file()?;
                }

                println!("{}", object.get_hash());
            }

            _ => println!("Unsupported command: {}", command),
        }

        Ok(())
    }
}
