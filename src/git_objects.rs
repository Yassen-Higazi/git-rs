use anyhow::bail;

use crate::utils::decompress;

pub enum GitObject {
    Blob(String, String),

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
            GitObject::Blob(_, _) => object_name = "blob",
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

        if content.starts_with("blob") {
            return Ok(GitObject::Blob(hash, content));
        }

        if content.starts_with("tree") {
            return Ok(GitObject::Tree {
                hash,
                objects: vec![],
            });
        }

        bail!("Unsupported Type")
    }

    pub fn print_content(&self) {
        match self {
            GitObject::Blob(_, content) => print!("{content}"),

            GitObject::Tree { hash, objects } => {
                for object in objects {
                    object.print_content();
                }
            }

            GitObject::Commit { .. } => todo!(),
        }
    }
}
