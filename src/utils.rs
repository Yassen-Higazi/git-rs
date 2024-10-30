use anyhow::{bail, Context};
use flate2::write::{ZlibDecoder, ZlibEncoder};
use flate2::Compression;
use sha1::{Digest, Sha1};
use std::fs;
use std::io::Write;
use std::num::ParseIntError;

pub fn to_hex_string(content: &[u8]) -> String {
    content
        .iter()
        .map(|b| format!("{:02x}", b).to_string())
        .collect::<Vec<String>>()
        .join("")
}

pub fn from_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect()
}

pub fn generate_object_id(content: &[u8]) -> anyhow::Result<String> {
    let result = Sha1::digest(content).to_vec();

    let result_str = to_hex_string(result.as_slice());

    Ok(result_str)
}

pub fn compress(bytes: &[u8]) -> anyhow::Result<Vec<u8>> {
    let mut e = ZlibEncoder::new(Vec::new(), Compression::default());

    e.write_all(bytes)
        .with_context(|| "Could not write bytes to ZlibEncoder")?;

    let compressed = e.finish().with_context(|| "Could not compress Object")?;

    Ok(compressed)
}

pub fn decompress(bytes: &[u8]) -> anyhow::Result<Vec<u8>> {
    let mut writer = Vec::new();

    let mut z = ZlibDecoder::new(writer);

    z.write_all(bytes)
        .with_context(|| "Could not write bytes to ZlibDecoder")?;

    writer = z.finish().with_context(|| "Could not decompress Object")?;

    // println!("Decompressed data: {:?}", writer);

    Ok(writer)
}

pub fn create_object_directory(hash: &str) -> anyhow::Result<String> {
    let (dir_name, file_name) = hash.split_at(2);

    let dir_path = format!(".git/objects/{dir_name}");

    let file_path = format!("{dir_path}/{file_name}");

    create_directory(dir_path.as_str())?;

    Ok(file_path)
}

pub fn create_directory(dir_name: &str) -> anyhow::Result<bool> {
    let res = fs::create_dir(dir_name);

    match res {
        Ok(_) => Ok(true),

        Err(e) => match e.kind() {
            std::io::ErrorKind::AlreadyExists => Ok(false),

            _ => bail!("{e}"),
        },
    }
}

pub fn write_to_file(file_name: &str, content: &[u8]) -> anyhow::Result<()> {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(file_name)?;

    file.write_all(content)?;

    Ok(())
}

pub fn read_object(hash: &str) -> anyhow::Result<Vec<u8>> {
    let (folder_name, file_name) = hash.split_at(2);

    let file_path = format!(".git/objects/{}/{}", folder_name, file_name);

    read_file(&file_path).with_context(|| format!("Could not read object at path: {file_path:?}"))
}

pub fn read_file(file_name: &str) -> anyhow::Result<Vec<u8>> {
    fs::read(file_name).with_context(|| format!("Could not read file: {file_name}"))
}

pub fn list_directory(dir_name: &str) -> anyhow::Result<Vec<fs::DirEntry>> {
    let paths = fs::read_dir(dir_name)?;

    let mut entries = Vec::new();

    for path in paths {
        if let Ok(elm) = path {
            entries.push(elm);
        } else {
            continue;
        }
    }

    Ok(entries)
}

pub fn get_hidden_files() -> anyhow::Result<Vec<String>> {
    let gitingore_result = read_file(".gitignore");

    let hidden_files = match gitingore_result {
        Ok(gitignore_buff) => {
            let hidden_files_str = String::from_utf8(gitignore_buff)?;

            let mut hidden_files: Vec<String> = hidden_files_str
                .split("\n")
                .map(|f| f.to_string())
                .collect();

            hidden_files.push(".git".to_string());

            hidden_files
        }

        Err(err) => {
            let err_str = err.to_string();

            if err_str.contains(".gitignore") {
                Vec::new()
            } else {
                bail!(err)
            }
        }
    };

    Ok(hidden_files)
}

pub fn filter_hidden_files(files: &[fs::DirEntry]) -> anyhow::Result<Vec<&fs::DirEntry>> {
    let hidden_files = get_hidden_files()?;

    let allowed_files: Vec<&fs::DirEntry> = files
        .iter()
        .filter(|entry| {
            let file_name = entry.file_name().to_str().unwrap_or(".git").to_string();

            if hidden_files.is_empty() {
                file_name != ".git"
            } else {
                file_name != ".git" || !hidden_files.contains(&file_name)
            }
        })
        .collect();

    Ok(allowed_files)
}
