use sha1::{Digest, Sha1};
use std::io;
use std::{
    fs,
    path::{Path, PathBuf},
};

fn get_objects_path() -> Result<PathBuf, io::Error> {
    let rit_dir = Path::new(".rit");
    let parent_dir = rit_dir.join("objects");

    if !parent_dir.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "No .rit directory found",
        ));
    }

    Ok(parent_dir)
}

fn get_hash_from_file(content: &Vec<u8>) -> String {
    let mut hasher = Sha1::new();
    hasher.update(content);
    format!("{:x}", hasher.finalize())
}

fn save_file_hash(file_hash: &String, parent_dir: &PathBuf, content: &Vec<u8>) -> io::Result<()> {
    let folder_name = &file_hash[..2];
    let file_name = &file_hash[3..];

    let path_name = Path::join(&parent_dir, folder_name);
    fs::create_dir(&path_name)?;

    let final_path = Path::join(&path_name, file_name);

    fs::write(final_path, &content)
}

pub fn add_rit(paths: Vec<&PathBuf>) -> Result<bool, Box<dyn std::error::Error>> {
    let parent_dir;

    match get_objects_path() {
        Ok(dir) => parent_dir = dir,
        Err(e) => return Err(Box::new(e)),
    }

    let mut success: usize = 0;
    for path in &paths {
        if !path.exists() {
            println!("{path:?} does NOT exist; skipping...");
            continue;
        }

        if path.is_file() {
            let content = fs::read(path)?;
            let file_hash = get_hash_from_file(&content);

            match save_file_hash(&file_hash, &parent_dir, &content) {
                Ok(_) => success += 1,
                Err(_) => {}
            }
        } else {
            println!("directory not supported for now");
        }
    }

    if success != paths.len() {
        Err("unsuccessful add operation".into())
    } else {
        Ok(true)
    }
}
