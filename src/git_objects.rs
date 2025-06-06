use std::fs;

use anyhow::bail;

use crate::utils::{
    compress, create_object_directory, decompress, filter_hidden_files, from_hex,
    generate_object_id, list_directory, read_file, read_object, to_hex_string, write_to_file,
};

#[derive(Debug)]
pub struct TreeObject {
    pub hash: String,

    pub name: String,

    pub mode: TreeFileModes,

    object_type: String,

    pub git_object: GitObject,
}

impl TreeObject {
    pub fn new(
        hash: String,
        name: String,
        mode: TreeFileModes,
        git_object: GitObject,
    ) -> TreeObject {
        Self {
            hash,
            name,
            object_type: match &mode {
                TreeFileModes::Directory => "tree".to_string(),

                _ => "blob".to_string(),
            },
            mode,
            git_object,
        }
    }
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

impl From<fs::FileType> for TreeFileModes {
    fn from(value: fs::FileType) -> Self {
        if value.is_dir() {
            Self::Directory
        } else if value.is_symlink() {
            Self::SymbolicLink
        } else {
            Self::Regular
        }
    }
}

impl std::fmt::Display for TreeFileModes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            TreeFileModes::Regular => "100644",
            TreeFileModes::Executable => "100755",
            TreeFileModes::SymbolicLink => "120000",
            TreeFileModes::Directory => "40000",
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
        message: String,
        tree: Box<GitObject>,
        parent: Option<Vec<GitObject>>,

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
    pub fn new_commit(
        message: &str,
        hash: &str,
        tree: GitObject,
        parent: Option<Vec<GitObject>>,
    ) -> Self {
        let username = String::from("Yassen Higazi");
        let email = String::from("yassenka28@gmail.com");

        GitObject::Commit {
            parent,
            tree: Box::new(tree),
            hash: hash.to_string(),
            message: message.to_string(),
            author_name: username.clone(),
            author_email: email.clone(),
            committer_name: username,
            committer_email: email,
        }
    }

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

                    let git_object_content = read_object(hash_str.as_str())?;

                    let git_object =
                        GitObject::from_file_content(hash_str.clone(), git_object_content)?;

                    let object = TreeObject {
                        hash: hash_str,
                        name: filename_str.to_string(),
                        object_type: match mode_enum {
                            TreeFileModes::Directory => "tree".to_string(),

                            _ => "blob".to_string(),
                        },
                        git_object,
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

            "commit" => {
                let hash = GitObject::get_or_generate_hash(obj_type, hash, content)?;

                let content_str = String::from_utf8(content.to_vec())?;

                let content_split: Vec<&str> = content_str.split("\n").collect();

                let tree_line: Vec<&str> = content_split[0].split(" ").collect();

                let tree_hash = tree_line[1];

                let tree_object_content = read_object(tree_hash)?;

                let tree_object =
                    GitObject::from_file_content(tree_hash.to_string(), tree_object_content)?;

                let mut line_index = 1;

                #[allow(unused_assignments)]
                let mut message = String::new();

                let mut parents: Vec<GitObject> = vec![];

                let mut author_name = String::new();

                let mut author_email = String::new();

                let mut committer_name = String::new();

                let mut committer_email = String::new();

                loop {
                    let current_line = content_split[line_index];

                    if current_line.starts_with("parent") {
                        let parent_line: Vec<&str> = current_line.split(" ").collect();

                        let parent_hash = parent_line[1];

                        let parent_object_content = read_object(parent_hash)?;

                        let parent_object = GitObject::from_file_content(
                            parent_hash.to_string(),
                            parent_object_content,
                        )?;

                        parents.push(parent_object);
                    } else if current_line.starts_with("author") {
                        let author_line: Vec<&str> = current_line.split(" ").collect();

                        author_name = author_line[1].to_string();

                        author_email = author_line[2].to_string().replace("<", "").replace(">", "");
                    } else if current_line.starts_with("committer") {
                        let committer_line: Vec<&str> = current_line.split(" ").collect();

                        committer_name = committer_line[1].to_string();

                        committer_email = committer_line[2]
                            .to_string()
                            .replace("<", "")
                            .replace(">", "");
                    } else {
                        message = content_split[line_index + 1].to_string();

                        break;
                    }

                    line_index += 1;
                }

                Ok(GitObject::Commit {
                    hash,
                    message,
                    author_name,
                    author_email,
                    committer_name,
                    committer_email,
                    tree: Box::new(tree_object),
                    parent: if parents.is_empty() {
                        None
                    } else {
                        Some(parents)
                    },
                })
            }

            _ => bail!("Unsupported Type"),
        }
    }

    pub fn from_directory(dir_path: &str) -> anyhow::Result<Self> {
        let all_files = list_directory(dir_path)?;

        let mut files = filter_hidden_files(&all_files)?;

        files.sort_by_key(|entry| {
            entry
                .file_name()
                .to_str()
                .expect("Could not get file name to sort directories")
                .to_owned()
        });

        let mut objects = Vec::new();

        let mut objects_vec = Vec::new();

        for entry in files {
            let file_path = entry.path();

            let path_str = file_path.to_str().expect("Could not get path string");

            let file_type = entry.file_type()?;

            let file_name = entry
                .file_name()
                .to_str()
                .expect("Could not get file name")
                .to_owned();

            let git_object = if file_type.is_dir() {
                GitObject::from_directory(path_str)?
            } else {
                let content = read_file(path_str)?;

                GitObject::from_file_content_and_type("blob", content.as_slice(), None)?
            };

            let object = TreeObject::new(
                git_object.get_hash().to_string(),
                file_name,
                TreeFileModes::from(file_type),
                git_object,
            );

            let object_buf = [
                format!("{} {}\0", object.mode, object.name).as_bytes(),
                from_hex(object.hash.as_str())?.as_slice(),
            ]
            .concat();

            objects_vec.push(object_buf);

            objects.push(object);
        }

        let objects_buffer = objects_vec.concat();

        let tree_size = objects_buffer.len() as u64;

        let final_content = [
            format!("tree {}\0", tree_size).as_bytes(),
            objects_buffer.as_slice(),
        ]
        .concat();

        Ok(GitObject::Tree {
            objects,
            size: tree_size,
            hash: generate_object_id(final_content.as_slice())?,
        })
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

            GitObject::Commit {
                message,
                tree,
                parent,
                author_name,
                author_email,
                committer_name,
                committer_email,
                ..
            } => {
                println!("tree {}", tree.get_hash());

                if let Some(parents) = parent {
                    for parent in parents {
                        println!("parent {}", parent.get_hash());
                    }
                }

                println!("author {author_name} <{author_email}> 1730371859 +0300");

                println!("committer {committer_name} <{committer_email}> 1730371859 +0300\n");

                println!("{message}");
            }
        }
    }

    pub fn print_type(&self) {
        print!("{}", self);
    }

    pub fn get_type(&self) -> String {
        format!("{}", self)
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

                let mut objects_vec = vec![format!("tree {size}\0").as_bytes().to_vec()];

                for object in objects {
                    object.git_object.write_to_file()?;

                    let object_buf = [
                        format!("{} {}\0", object.mode, object.name).as_bytes(),
                        from_hex(object.hash.as_str())?.as_slice(),
                    ]
                    .concat();

                    objects_vec.push(object_buf);
                }

                let final_content = compress(objects_vec.concat().as_slice())?;

                write_to_file(path.as_str(), final_content.as_slice())?;

                Ok(())
            }

            GitObject::Commit {
                hash,
                tree,
                parent,
                message,
                author_name,
                author_email,
                committer_name,
                committer_email,
            } => {
                let path = create_object_directory(hash)?;

                let mut content: Vec<Vec<u8>> = vec![
                    b"tree ".to_vec(),
                    tree.get_hash().as_bytes().to_vec(),
                    b"\n".to_vec(),
                ];

                let mut parents: Vec<Vec<u8>> = vec![];

                if let Some(parent_commits) = parent {
                    for commit in parent_commits {
                        parents.push(b"parent ".to_vec());

                        parents.push(commit.get_hash().as_bytes().to_vec());

                        parents.push(b"\n".to_vec());
                    }
                }

                content.push(parents.concat());

                content.push(
                    format!("author {author_name} <{author_email}>")
                        .as_bytes()
                        .to_vec(),
                );

                content.push(b"\n".to_vec());

                content.push(
                    format!("committer {committer_name} <{committer_email}>")
                        .as_bytes()
                        .to_vec(),
                );

                content.push(b"\n\n".to_vec());

                content.push(message.as_bytes().to_vec());

                content.push(b"\n".to_vec());

                let uncomposed_content = content.concat();

                let final_content = [
                    format!("commit {}\0", uncomposed_content.len()).as_bytes(),
                    uncomposed_content.as_slice(),
                ]
                .concat();

                let compressed_content = compress(&final_content)?;

                write_to_file(path.as_str(), compressed_content.as_slice())?;

                Ok(())
            }
        }
    }

    pub fn get_hash(&self) -> &String {
        match self {
            GitObject::Blob { hash, .. } => hash,
            GitObject::Tree { hash, .. } => hash,
            GitObject::Commit { hash, .. } => hash,
        }
    }

    pub fn is_tree(&self) -> bool {
        matches!(self, GitObject::Tree { .. })
    }

    pub fn is_commit(&self) -> bool {
        matches!(self, GitObject::Commit { .. })
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
