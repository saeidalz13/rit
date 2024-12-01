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

fn write_tree_file(objects_path: &PathBuf) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
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

    let (tree_file_name, tree_file_hash) = get_hash_from_file(&tree_content);
    ioutils::save_file_hash(&tree_file_name, objects_path, &tree_content)?;

    Ok(tree_file_hash)
}

fn write_commit_file(
    objects_path: &PathBuf,
    parent_commit_hash: Vec<u8>,
    tree_file_hash: Vec<u8>,
    commit_msg: &str,
) -> Result<Vec<u8>, std::io::Error> {
    let mut commit_content: Vec<u8> = Vec::new();

    // 4 bytes
    commit_content.extend_from_slice(b"tree");

    // 32 bytes
    commit_content.extend_from_slice(&tree_file_hash[..]);

    // 32 bytes if available
    if !parent_commit_hash.is_empty() {
        commit_content.extend_from_slice(&parent_commit_hash[..]);
    }

    // unknown number of bytes, read until 10
    commit_content.extend_from_slice(b"author saeid saeidalz96@gmail.com\n");

    // unknown number of bytes, read until 10
    commit_content.extend_from_slice(b"committer saeid saeidalz96@gmail.com\n");

    // one byte for space
    commit_content.push(b' ');
    // unknows bytes, read until end for msg
    commit_content.extend_from_slice(commit_msg.as_bytes());

    let (commit_file_name, commit_file_hash) = get_hash_from_file(&commit_content);
    ioutils::save_file_hash(&commit_file_name, &objects_path, &commit_content)?;

    Ok(commit_file_hash)
}

fn fetch_parent_commit_hash(main_file: &Path) -> Result<(Vec<u8>, bool), std::io::Error> {
    let mut parent_commit_hash: Vec<u8> = Vec::new();
    let mut main_file_exists = false;
    if main_file.exists() {
        parent_commit_hash = fs::read(main_file)?;
        main_file_exists = true;
    }

    Ok((parent_commit_hash, main_file_exists))
}

fn write_head_commit(main_file_exists: bool, main_file: &Path, commit_file_hash: Vec<u8>) {
    let mut f: fs::File;
    if main_file_exists {
        f = fs::OpenOptions::new().write(true).open(main_file).unwrap();
    } else {
        fs::create_dir_all(main_file.parent().unwrap()).unwrap();
        f = fs::File::create(main_file).unwrap();
    }
    f.write(&commit_file_hash[..]).unwrap();
}

pub fn commit_rit(commit_msg: &str) {
    let objects_path = ioutils::get_objects_path().unwrap();
    let main_file = Path::new("./.rit/refs/heads/main");

    let tree_file_hash: Vec<u8>;
    match write_tree_file(&objects_path) {
        Ok(p) => tree_file_hash = p,

        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    }

    let (parent_commit_hash, main_file_exists) = fetch_parent_commit_hash(main_file).unwrap();

    let commit_file_hash = write_commit_file(
        &objects_path,
        parent_commit_hash,
        tree_file_hash,
        commit_msg,
    )
    .unwrap();

    write_head_commit(main_file_exists, main_file, commit_file_hash)
}
