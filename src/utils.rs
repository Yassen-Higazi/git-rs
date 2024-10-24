use anyhow::{bail, Context};
use flate2::write::{ZlibDecoder, ZlibEncoder};
use flate2::Compression;
use sha1::{Digest, Sha1};
use std::fs;
use std::io::Write;

pub fn to_hex_string(content: &[u8]) -> String {
    content
        .iter()
        .map(|b| format!("{:02x}", b).to_string())
        .collect::<Vec<String>>()
        .join("")
}

pub fn generate_object_id(content: &[u8]) -> anyhow::Result<String> {
    let result = Sha1::digest(content).to_vec();

    let result_str = to_hex_string(result.as_slice());

    Ok(result_str)
}

pub fn compress(bytes: &[u8]) -> anyhow::Result<Vec<u8>> {
    let mut e = ZlibEncoder::new(Vec::new(), Compression::default());

    e.write_all(bytes)?;

    let compressed = e.finish()?;

    Ok(compressed)
}

pub fn decompress(bytes: &[u8]) -> anyhow::Result<Vec<u8>> {
    let mut writer = Vec::new();

    let mut z = ZlibDecoder::new(writer);

    z.write_all(bytes)?;

    writer = z.finish()?;

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
