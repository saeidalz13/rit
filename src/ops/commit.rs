use crate::utils::{hashutils::get_hash_from_file, ioutils};
use hex;
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};
/// This module takes care of 'rit commit'.
/// To achieve this we need to:
///
/// 1. Have a commit file which contains
/// tree <TREE_HASH>
/// parent <COMMIT_HASH>  (Omit this line if it's the first commit)
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

const ASCII_CHAR_SPACE: u8 = 32;
const ASCII_CHAR_NEWLINE: u8 = 10;

fn create_tree_file(objects_path: &PathBuf) -> Result<String, Box<dyn std::error::Error>> {
    let (_, ies) = ioutils::read_index()?;

    let mut tree_content: Vec<u8> = Vec::new();

    for ie in ies.iter() {
        // 4 bytes for mode
        tree_content.extend_from_slice(&ie.mode.to_be_bytes());
        // 4 bytes for file_path_len
        tree_content.extend_from_slice(&ie.file_path_len.to_be_bytes());
        // unknows bytes for file_path (inferred from the previous 4 bytes)
        tree_content.extend_from_slice(ie.file_path.as_bytes());
        // 32 bytes for sha hash
        tree_content.extend_from_slice(&ie.sha_hash[..]);

        // Adding extra 0 if necessary for 8-byte alignment
        let offset = 8 - (tree_content.len() % 8);
        if offset > 0 {
            tree_content.extend_from_slice(&vec![0; offset]);
        }
    }

    let (tree_file_name, _) = get_hash_from_file(&tree_content);
    ioutils::save_file_hash(&tree_file_name, objects_path, &tree_content)?;

    Ok(tree_file_name)
}

// fn create_commit_file() {}

pub fn commit_rit(commit_msg: &str) {
    let objects_path = ioutils::get_objects_path().unwrap();

    let tree_file_name: String;
    match create_tree_file(&objects_path) {
        Ok(p) => tree_file_name = p,

        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    }

    let mut parent_commit_hash = String::new();
    let main_file = Path::new("./.rit/refs/heads/main");
    let mut main_file_exists = false;
    if main_file.exists() {
        parent_commit_hash = fs::read_to_string(main_file).unwrap();
        main_file_exists = true;
    }

    let mut commit_content: Vec<u8> = Vec::new();

    commit_content.extend_from_slice(b"tree ");
    commit_content.extend_from_slice(tree_file_name.as_bytes());
    commit_content.push(ASCII_CHAR_NEWLINE);

    if !parent_commit_hash.is_empty() {
        commit_content.extend_from_slice(parent_commit_hash.as_bytes());
        commit_content.push(ASCII_CHAR_NEWLINE);
    }

    // todo!("add date and permission for both author and committer");
    commit_content.extend_from_slice(b"author saeid saeidalz96@gmail.com\n");
    commit_content.extend_from_slice(b"committer saeid saeidalz96@gmail.com\n");

    commit_content.push(ASCII_CHAR_NEWLINE);
    commit_content.extend_from_slice(commit_msg.as_bytes());

    let (commit_file_name, _) = get_hash_from_file(&commit_content);
    ioutils::save_file_hash(&commit_file_name, &objects_path, &commit_content).unwrap();

    let mut f: fs::File;
    if main_file_exists {
        f = fs::OpenOptions::new().write(true).open(main_file).unwrap();
    } else {
        fs::create_dir_all(main_file.parent().unwrap()).unwrap();
        f = fs::File::create(main_file).unwrap();
    }
    f.write(commit_file_name.as_bytes()).unwrap();
}
