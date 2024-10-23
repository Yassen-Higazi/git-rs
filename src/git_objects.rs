use anyhow::bail;

use crate::utils::{compress, create_directory, decompress, generate_object_id, write_to_file};

#[derive(Debug)]
#[allow(dead_code)]
pub enum GitObject {
    Blob {
        hash: String,
        size: u64,
        content: String,
    },

    Tree {
        hash: String,
        objects: Vec<GitObject>,
    },

    Commit {
        hash: String,
        tree: Box<GitObject>,
        parent: Box<GitObject>,

        author_name: String,
        author_email: String,
        committer_name: String,
        committer_email: String,
    },
}

impl std::fmt::Display for GitObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[allow(unused_assignments)]
        let mut object_name: &str = "";

        match self {
            GitObject::Blob { .. } => object_name = "blob",
            GitObject::Tree { .. } => object_name = "tree",
            GitObject::Commit { .. } => object_name = "commit",
        }

        write!(f, "{object_name}")
    }
}

impl GitObject {
    pub fn from_file_content(
        hash: String,
        compressed_content: Vec<u8>,
    ) -> anyhow::Result<GitObject> {
        let content = decompress(&compressed_content)?;

        let obj_type = &content[0..4];

        let arr = content.split("\0").collect::<Vec<&str>>();

        let final_content = arr[1..].join("");

        GitObject::from_file_content_and_type(obj_type, final_content, Some(hash))
    }

    pub fn from_file_content_and_type(
        obj_type: &str,
        content: String,
        hash: Option<String>,
    ) -> anyhow::Result<GitObject> {
        match obj_type {
            "blob" => {
                let object_hash: String = match hash {
                    Some(hash) => hash,

                    None => {
                        let hash_content = format!("blob {}\0{}", content.len(), content);

                        generate_object_id(hash_content.as_bytes())?
                    }
                };

                Ok(GitObject::Blob {
                    hash: object_hash,
                    size: content.len() as u64,
                    content,
                })
            }

            _ => bail!("Unsupported Type"),
        }
    }

    pub fn print_content(&self) {
        match self {
            GitObject::Blob { content, .. } => print!("{content}"),

            GitObject::Tree { objects, .. } => {
                for object in objects {
                    // TODO: fix: print summary
                    object.print_content();
                }
            }

            GitObject::Commit { .. } => todo!(),
        }
    }

    pub fn print_type(&self) {
        print!("{}", self);
    }

    pub fn print_size(&self) -> anyhow::Result<()> {
        match self {
            GitObject::Blob { size, .. } => print!("{size}"),

            _ => bail!("Not Implemented"),
        };

        Ok(())
    }

    pub fn write_to_file(&self) -> anyhow::Result<()> {
        match self {
            GitObject::Blob {
                hash,
                size,
                content,
            } => {
                let (dir_name, file_name) = hash.split_at(2);

                let dir_path = format!(".git/objects/{dir_name}");

                let file_path = format!("{dir_path}/{file_name}");

                create_directory(dir_path.as_str())?;

                let final_content = format!("blob {size}\0{content}");

                let compressed_content = compress(final_content.as_bytes())?;

                write_to_file(file_path.as_str(), compressed_content.as_slice())?;

                Ok(())
            }

            _ => bail!("Not Implemented"),
        }
    }

    pub fn get_hash(&self) -> &String {
        match self {
            GitObject::Blob { hash, .. } => hash,
            GitObject::Tree { hash, .. } => hash,
            GitObject::Commit { hash, .. } => hash,
        }
    }
}
