use log::{error, info};
use serde::{Deserialize, Serialize};
use std::{
    fs::{create_dir_all, read_dir, File},
    io::{Read, Write},
    path::{Path, PathBuf},
    time::SystemTime,
};

use fast_rsync::{Signature, apply, diff};

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Data {
    WebMessage { directories: Vec<Directory> },
    JsonFileMessage { file: FileData },
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Directory {
    pub file_path: PathBuf,
    pub modified: SystemTime,
    pub size: u64,
}

impl Directory {
    fn new(file_path: PathBuf, modified: SystemTime, size: u64) -> Self {
        Self {
            file_path,
            modified,
            size,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct FileData {
    pub dir: PathBuf,
    pub contents: Vec<u8>,
}

impl FileData {
    fn new(dir: PathBuf, contents: Vec<u8>) -> Self {
        Self { dir, contents }
    }
}

pub fn walk_dir(dir: &PathBuf) -> Result<Vec<Directory>, Box<dyn std::error::Error>> {
    let mut res: Vec<Directory> = Vec::new();

    if dir.is_dir() {
        for entry in read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            let metadata = entry.metadata()?;
            let size = metadata.len();
            let modified = metadata.modified()?;

            if metadata.is_dir() {
                let sub_dir_entries = walk_dir(&path)?;
                res.extend(sub_dir_entries);
            } else {
                let t = Directory::new(path, modified, size);
                res.push(t);
            }
        }
    } else {
        error!("{} is not a directory", dir.to_string_lossy());
    }

    Ok(res)
}

pub fn transfer_new_file(file: &Directory) -> Result<FileData, Box<dyn std::error::Error>> {
    let file_path = file.file_path.clone();
    let mut open_file = File::open(&file.file_path)?;
    let mut contents: Vec<u8> = Vec::new();
    open_file.read_to_end(&mut contents)?;

    let t = FileData::new(file_path, contents);

    Ok(t)
}

fn find_common_prefix<'a>(path1: &'a Path, path2: &'a Path) -> (PathBuf, &'a Path) {
    let mut common_prefix = PathBuf::new();
    let mut iter1 = path1.components();
    let mut iter2 = path2.components();

    loop {
        match (iter1.next(), iter2.next()) {
            (Some(comp1), Some(comp2)) if comp1 == comp2 => {
                common_prefix.push(comp1);
            }
            _ => break,
        }
    }

    (common_prefix, iter1.as_path())
}

pub fn replace_files(file_data: FileData, dest: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let path1: &Path = file_data.dir.as_path();
    let path2: &Path = dest.as_path();

    let (common_prefix, relative_path1) = find_common_prefix(path1, path2);

    let relative_path2 = path2.strip_prefix(common_prefix.clone())?;

    let combined_path = common_prefix.join(relative_path2).join(relative_path1);

    if let Some(parent_dir) = combined_path.parent() {
        create_dir_all(parent_dir)?;
    }

    let mut file = File::create(&combined_path)?;
    file.write_all(&file_data.contents)?;

    Ok(())
}

pub fn find_changed_files(
    client_dir: Vec<Directory>,
    server_dir: PathBuf,
) -> Result<Vec<Directory>, Box<dyn std::error::Error>> {
    let dir = walk_dir(&server_dir)?;

    info!("server_dir: {:?}", dir.len());
    info!("client_dir: {:?}", client_dir.len());

    let mut changed_files: Vec<Directory> = Vec::new();

    for client_file in client_dir.iter() {
        if let Some(server_file) = dir.iter().find(|&file| {
            let fs = file.file_path.file_name();
            let cs = client_file.file_path.file_name();
            fs == cs
        }) {
            if (client_file.modified != server_file.modified)
                && (client_file.size != server_file.size)
            {
                changed_files.push(client_file.clone());
            }
        } else {
            changed_files.push(client_file.clone());
        }
    }

    info!("{:?} to be copied over", changed_files.len());

    Ok(changed_files)
}
