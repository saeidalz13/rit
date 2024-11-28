use sha2::{Digest, Sha256};
use std::io::{self, ErrorKind};
use std::os::unix::fs::MetadataExt;
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::utils::hashutils::get_hash_from_file;
use crate::utils::ioutils::{add_index, read_index, IndexEntry, IndexHeader};

fn get_objects_path() -> Result<PathBuf, io::Error> {
    let rit_dir = Path::new(".rit");
    let objects_path = rit_dir.join("objects");

    if !objects_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "No .rit directory found",
        ));
    }

    Ok(objects_path)
}

fn save_file_hash(file_hash: &String, objects_path: &PathBuf, content: &Vec<u8>) -> io::Result<()> {
    let folder_name = &file_hash[..2];
    let file_name = &file_hash[2..];

    let path_name = Path::join(&objects_path, folder_name);
    fs::create_dir(&path_name)?;

    let final_path = Path::join(&path_name, file_name);

    fs::write(final_path, &content)
}

fn is_path_processable(path: &PathBuf) -> bool {
    if path.is_dir() {
        println!("{} is a dir. Choose file please.", path.display());
        return false;
    }

    if !path.exists() {
        println!("{} does NOT exist; skipping...", path.display());
        return false;
    }

    true
}

fn get_file_path_info(path: &PathBuf) -> (String, u32) {
    let file_path;
    if !path.is_absolute() {
        file_path = format!("{}", Path::new(".").join(path).to_string_lossy());
    } else {
        file_path = format!("{}", path.to_string_lossy());
    }

    let file_path_len = file_path.as_bytes().len() as u32;
    (file_path, file_path_len)
}

/// This should add the series of files requested by user.
/// First it checks what files exist if .rit/INDEX exists.
/// If the required files already added, then it does nothing.
/// Any new requested files will be added.
pub fn add_rit(paths: Vec<&PathBuf>) -> Result<bool, Box<dyn std::error::Error>> {
    let objects_path: PathBuf;
    match get_objects_path() {
        Ok(dir) => objects_path = dir,
        Err(e) => return Err(Box::new(e)),
    }

    let mut index_entries: Vec<IndexEntry> = vec![];
    let mut header = IndexHeader::new(0, 3, *b"DIRC");
    let mut existing_paths = vec![];
    match read_index() {
        Ok(res) => {
            header = res.0;
            index_entries = res.1;

            for ie in &index_entries {
                existing_paths.push(ie.file_path.clone());
            }
        }
        Err(e) => {
            if e.kind() != ErrorKind::NotFound {
                return Err(e.into());
            }
        }
    }

    let all_paths_len = paths.len();
    let mut success: usize = 0;
    for path in paths.into_iter() {
        if !is_path_processable(path) {
            continue;
        }

        let (file_path, file_path_len) = get_file_path_info(path);

        if existing_paths.contains(&file_path) {
            println!("this file already added.");
            continue;
        }

        let content = fs::read(path)?;
        let (file_hash, hash_vec) = get_hash_from_file(&content);

        match save_file_hash(&file_hash, &objects_path, &content) {
            Ok(_) => {}
            Err(e) => {
                println!("hash save error: {}", e);
                continue;
            }
        }

        let md;
        match fs::metadata(path) {
            Ok(m) => md = m,
            Err(e) => {
                println!("reading metadata error: {}", e);
                continue;
            }
        }

        index_entries.push(IndexEntry {
            ctime: (md.ctime() as u32, md.ctime_nsec() as u32),
            mtime: (md.mtime() as u32, md.mtime_nsec() as u32),
            device: md.dev() as u32,
            inode: md.ino() as u32,
            mode: md.mode() as u32,
            size: md.size() as u32,
            sha_hash: hash_vec,
            file_path_len,
            file_path,
        });
        header.increment_num_entries();
        success += 1;
    }

    if success > 0 {
        add_index(header, index_entries)?;
    }

    if success != all_paths_len {
        Err("unsuccessful add operation".into())
    } else {
        Ok(true)
    }
}
