use crate::utils::{hashutils::get_hash_from_file, ioutils};
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
        // should be octal mode of permission
        // let mode = u8::from(&ie.mode).try_into()?;
        tree_content.extend_from_slice(&ie.mode.to_be_bytes());
        tree_content.push(ASCII_CHAR_SPACE);

        tree_content.extend_from_slice(ie.file_path.as_bytes());
        tree_content.push(ASCII_CHAR_SPACE);

        let file_content = fs::read(&ie.file_path)?;
        let (_, hash_value) = get_hash_from_file(&file_content);

        tree_content.extend_from_slice(&hash_value);
        tree_content.push(ASCII_CHAR_NEWLINE); // new line
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
    commit_content.extend_from_slice(b"author ");
    commit_content.extend_from_slice(b"saeid ");
    commit_content.extend_from_slice(b"saeidalz96@gmail.com ");
    commit_content.push(ASCII_CHAR_NEWLINE);

    commit_content.extend_from_slice(b"committer ");
    commit_content.extend_from_slice(b"saeid ");
    commit_content.extend_from_slice(b"saeidalz96@gmail.com ");
    commit_content.push(ASCII_CHAR_NEWLINE);

    commit_content.push(ASCII_CHAR_NEWLINE);
    commit_content.extend_from_slice(commit_msg.as_bytes());

    let (commit_file_name, commit_file_hash) = get_hash_from_file(&commit_content);
    ioutils::save_file_hash(&commit_file_name, &objects_path, &commit_content).unwrap();

    let mut f: fs::File;
    if main_file_exists {
        f = fs::File::open(main_file).unwrap();
    } else {
        f = fs::OpenOptions::new().write(true).open(main_file).unwrap();
    }
    f.write(&commit_file_hash).unwrap();
}
