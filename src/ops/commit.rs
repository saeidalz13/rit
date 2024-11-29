use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::utils::{hashutils::get_hash_from_file, ioutils};

/// This module takes care of 'rit commit'.
/// To achieve this we need to:
///
/// 1. Have a commit file which contains
/// tree <TREE_HASH>
/// author <AUTHOR_NAME> <EMAIL> <DATE_SEC> <PERMISSION>
/// committer <COMMITTER_NAME> <EMAIL> <DATE_SEC> <PERMISSION>
///
/// <COMMIT_MSG>
///
/// 2. Have a tree file which contains
/// <MODE> <FILENAME> <HASH>
///
/// where <MODE> is permission in octal. (100644 normal files) (100755 executables) (040000 subdirs), etc.
///
///

// let mut paths: Vec<String> = vec![];
// for ie in ies.iter() {
//     paths.push(ie.file_path.clone());
// }
// This approach prevents cloning the strings
// let paths = ies.into_iter().map(|ie| ie.file_path).collect();
fn create_tree_file() -> Result<String, Box<dyn std::error::Error>> {
    let (_, ies) = ioutils::read_index()?;

    let space_ascii = 32 as u8;
    let newline_ascii = 10 as u8;
    let mut content: Vec<u8> = Vec::new();

    for ie in ies.iter() {
        // should be octal mode of permission
        // let mode = u8::from(&ie.mode).try_into()?;
        content.extend_from_slice(&ie.mode.to_be_bytes());
        content.push(space_ascii);

        content.extend_from_slice(ie.file_path.as_bytes());
        content.push(space_ascii);

        let file_content = fs::read(&ie.file_path)?;
        let (_, hash_value) = get_hash_from_file(&file_content);

        content.extend_from_slice(&hash_value);
        content.push(newline_ascii); // new line
    }

    let (tree_file_name, _) = get_hash_from_file(&content);
    let objects_path = ioutils::get_objects_path()?;
    ioutils::save_file_hash(&tree_file_name, &objects_path, &content)?;

    Ok(tree_file_name)
}

fn create_commit_file() {}

pub fn commit_rit(commit_msg: &String) {
    let tree_file_name: String;
    match create_tree_file() {
        Ok(p) => tree_file_name = p,

        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    }

    println!("Commit Message: {}", commit_msg);
    println!("{}", tree_file_name);
}
