use anyhow::{bail, ensure};

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
                object_type,
            } => {
                let (hash, object_type) = if let Some(h) = hash {
                    (h, object_type.clone())
                } else if let Some(obj_type) = object_type {
                    (obj_type, None)
                } else {
                    bail!("Invalid Command");
                };

                let compressed_content = read_object(hash.as_str())?;

                let object = GitObject::from_file_content(hash.to_owned(), compressed_content)?;

                if let Some(obj_type) = object_type {
                    ensure!(object.get_type() == obj_type.as_str(), "Invalid object");

                    object.print_content(false);
                } else if *pretty_print {
                    object.print_content(false);
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

                let object = GitObject::from_file_content_and_type(object_type, &content, None)?;

                if *write {
                    object.write_to_file()?;
                }

                println!("{}", object.get_hash());
            }

            Commands::LsTree { name_only, hash } => {
                let compressed_content = read_object(hash)?;

                let object = GitObject::from_file_content(hash.clone(), compressed_content)?;

                object.print_content(*name_only);
            }

            Commands::WriteTree => {
                let object = GitObject::from_directory(".")?;

                object.write_to_file()?;

                print!("{}", object.get_hash());
            }

            Commands::CommitTree {
                message,
                parent,
                tree,
            } => {
                let tree_content = read_object(tree)?;

                let tree_object = GitObject::from_file_content(tree.clone(), tree_content)?;

                ensure!(tree_object.is_tree(), "hash must be a tree object");

                let mut parents: Option<Vec<GitObject>> = None;

                if let Some(parent) = parent {
                    let parent_content = read_object(parent)?;

                    let parent_object =
                        GitObject::from_file_content(parent.clone(), parent_content)?;

                    ensure!(parent_object.is_commit(), "parent is not a commit object");

                    parents = Some(vec![parent_object])
                }

                let commit =
                    GitObject::new_commit(message.as_str(), tree.as_str(), tree_object, parents);

                commit.write_to_file()?;

                print!("{}", commit.get_hash());
            }

            _ => println!("Unsupported command: {}", command),
        }

        Ok(())
    }
}
