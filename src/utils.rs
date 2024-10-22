use anyhow::Context;
use flate2::write::ZlibDecoder;
use std::{fs, io::Write};

pub fn decompress(bytes: &Vec<u8>) -> anyhow::Result<String> {
    let mut writer = Vec::new();

    let mut z = ZlibDecoder::new(writer);

    z.write_all(&bytes[..])?;

    writer = z.finish()?;

    String::from_utf8(writer).with_context(|| "Could not decompress content")
}

pub fn create_directory(dir_name: &str) -> anyhow::Result<()> {
    fs::create_dir(dir_name).with_context(|| format!("Could not create directory {dir_name}"))
}

pub fn write_to_file(file_name: &str, content: &[u8]) -> anyhow::Result<()> {
    fs::write(file_name, content).with_context(|| format!("Could not write to file: {file_name}"))
}

pub fn read_object(hash: &str) -> anyhow::Result<Vec<u8>> {
    let (folder_name, file_name) = hash.split_at(2);

    let file_path = format!(".git/objects/{}/{}", folder_name, file_name);

    read_file(&file_path).with_context(|| format!("Could not read object at path: {file_path:?}"))
}

pub fn read_file(file_name: &str) -> anyhow::Result<Vec<u8>> {
    fs::read(file_name).with_context(|| format!("Could not read file: {file_name}"))
}
