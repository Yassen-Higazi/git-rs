use anyhow::bail;

use crate::utils::{
    compress, create_object_directory, decompress, generate_object_id, to_hex_string, write_to_file,
};

#[derive(Debug)]
pub struct TreeObject {
    pub hash: String,

    pub name: String,

    pub mode: TreeFileModes,

    object_type: String,
}

impl std::fmt::Display for TreeObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{} {} {}    {}",
            self.mode, self.object_type, self.hash, self.name
        )
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum TreeFileModes {
    Regular,

    Executable,

    SymbolicLink,

    Directory,
}

impl From<&str> for TreeFileModes {
    fn from(value: &str) -> Self {
        match value {
            "100644" => TreeFileModes::Regular,

            "100755" => TreeFileModes::Executable,

            "120000" => TreeFileModes::SymbolicLink,

            "040000" | "40000" => TreeFileModes::Directory,

            _ => TreeFileModes::Regular,
        }
    }
}

impl std::fmt::Display for TreeFileModes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            TreeFileModes::Regular => "100644",
            TreeFileModes::Executable => "100755",
            TreeFileModes::SymbolicLink => "120000",
            TreeFileModes::Directory => "040000",
        };

        write!(f, "{value}")
    }
}

#[derive(Debug)]
pub enum GitObject {
    Blob {
        hash: String,
        size: u64,
        content: String,
    },

    Tree {
        size: u64,
        hash: String,
        objects: Vec<TreeObject>,
    },

    #[allow(dead_code)]
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

        let (obj_type, final_content) = GitObject::parse_content_header(&content)?;

        GitObject::from_file_content_and_type(obj_type.as_str(), final_content, Some(hash))
    }

    pub fn from_file_content_and_type(
        obj_type: &str,
        content: &[u8],
        hash: Option<String>,
    ) -> anyhow::Result<GitObject> {
        match obj_type {
            "blob" => {
                let final_content = String::from_utf8(content.to_vec())?;

                Ok(GitObject::Blob {
                    content: final_content,
                    size: content.len() as u64,
                    hash: GitObject::get_or_generate_hash(obj_type, hash, content)?,
                })
            }

            "tree" => {
                let hash = GitObject::get_or_generate_hash(obj_type, hash, content)?;

                let mut objects = Vec::<TreeObject>::new();

                let mut iter = content.iter();

                while let Some(&byte) = iter.next() {
                    // Parse mode: Read bytes until space (' ')
                    let mut mode = Vec::new();

                    mode.push(byte);

                    for &b in iter.by_ref() {
                        if b == b' ' {
                            break;
                        }

                        mode.push(b);
                    }

                    let mode_str = String::from_utf8_lossy(&mode);

                    // Parse filename: Read bytes until null byte ('\0')
                    let mut filename = Vec::new();

                    for &b in iter.by_ref() {
                        if b == 0 {
                            break;
                        }

                        filename.push(b);
                    }

                    let filename_str = String::from_utf8_lossy(&filename);

                    // Parse the SHA-1 hash (next 20 bytes)
                    let sha1_hash: Vec<u8> = iter.by_ref().take(20).cloned().collect();

                    let hash_str = to_hex_string(sha1_hash.as_slice());

                    let mode_enum = TreeFileModes::from(mode_str.to_string().as_str());

                    let object = TreeObject {
                        hash: hash_str,
                        name: filename_str.to_string(),
                        object_type: match mode_enum {
                            TreeFileModes::Directory => "tree".to_string(),

                            _ => "blob".to_string(),
                        },

                        mode: mode_enum,
                    };

                    objects.push(object);
                }

                Ok(GitObject::Tree {
                    hash,
                    objects,
                    size: content.len() as u64,
                })
            }

            _ => bail!("Unsupported Type"),
        }
    }

    pub fn print_content(&self, name_only: bool) {
        match self {
            GitObject::Blob { content, .. } => print!("{content}"),

            GitObject::Tree { objects, .. } => {
                for object in objects {
                    if name_only {
                        println!("{}", object.name);
                    } else {
                        print!("{}", object);
                    }
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

            GitObject::Tree { size, .. } => print!("{size}"),

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
                let path = create_object_directory(hash)?;

                let final_content = format!("blob {size}\0{content}");

                let compressed_content = compress(final_content.as_bytes())?;

                write_to_file(path.as_str(), compressed_content.as_slice())?;

                Ok(())
            }

            GitObject::Tree {
                size,
                hash,
                objects,
            } => {
                let path = create_object_directory(hash)?;

                let mut objects_str = String::new();

                for object in objects {
                    objects_str.push_str(
                        format!("{} {}\0{}", object.mode, object.name, object.hash).as_str(),
                    )
                }

                let final_content = format!("tree {size}\0{objects_str}");

                write_to_file(path.as_str(), final_content.as_bytes())?;

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

    fn get_or_generate_hash(
        object_type: &str,
        hash: Option<String>,
        content: &[u8],
    ) -> anyhow::Result<String> {
        match hash {
            Some(hash) => Ok(hash),

            None => {
                let hash_header = format!("{object_type} {}\0", content.len());

                let hash_content = [hash_header.as_bytes(), content].concat();

                generate_object_id(hash_content.as_slice())
            }
        }
    }

    fn parse_content_header(content: &[u8]) -> anyhow::Result<(String, &[u8])> {
        let mut type_len = 4;
        let mut object_type = &content[0..type_len];

        if object_type == b"comm" {
            type_len = 6;
            object_type = &content[0..type_len];
        }

        let object_type_str = String::from_utf8(object_type.to_vec())?;

        let mut content_index = type_len + 1;

        while content_index < content.len() {
            if &content[content_index..content_index + 1] == b"\0" {
                content_index += 1;
                break;
            }

            content_index += 1;
        }

        let final_content = &content[content_index..];

        Ok((object_type_str, final_content))
    }
}
