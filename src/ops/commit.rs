use crate::{
    models::indexmodels::IndexEntry,
    utils::{hashutils::get_hash_from_file, ioutils},
};
use std::{
    fs, io,
    io::Write,
    path::{Path, PathBuf},
};

/// Tree file which contains:
/// - <MODE> <FILENAME> <HASH>
/// - where <MODE> is permission in octal. (100644 normal files) (100755 executables) (040000 subdirs), etc.
fn write_tree_file(objects_path: &PathBuf, ies: Vec<IndexEntry>) -> io::Result<Vec<u8>> {
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
        let offset = tree_content.len() % 8;
        if offset > 0 {
            let padding = 8 - offset;
            tree_content.extend_from_slice(&vec![0; padding]);
        }
    }

    let (tree_file_name, tree_file_hash) = get_hash_from_file(&tree_content);
    ioutils::save_file_hash(&tree_file_name, objects_path, &tree_content)?;

    Ok(tree_file_hash)
}

/// Commit file consists of:
/// - <TREE_HASH> (32 bytes)
/// - parent existence flag (1 byte)
/// - <COMMIT_HASH> (32 bytes)
/// - <AUTHOR_NAME> <EMAIL> <DATE_SEC> <PERMISSION> (variable bytes)
/// - com <COMMITTER_NAME> <EMAIL> <DATE_SEC> <PERMISSION> (variable bytes)
///
/// <COMMIT_MSG> (variable bytes)
fn prepare_commit_content(
    parent_commit_hash: &Vec<u8>,
    tree_file_hash: Vec<u8>,
    commit_msg: &str,
) -> Vec<u8> {
    let mut commit_content: Vec<u8> = Vec::new();

    // 32 bytes
    commit_content.extend_from_slice(&tree_file_hash[..]);

    // 33 bytes
    // 1 byte flag
    // 32 bytes hash sha256
    if !parent_commit_hash.is_empty() {
        commit_content.push(0b00000001);
        commit_content.extend_from_slice(&parent_commit_hash[..]);
    } else {
        commit_content.push(0b00000000);
        commit_content.extend_from_slice(&vec![0; 32]);
    }

    // unknown number of bytes, read until 10
    commit_content.extend_from_slice(b"saeid saeidalz96@gmail.com\n");

    // unknown number of bytes, read until 10
    commit_content.extend_from_slice(b"saeid saeidalz96@gmail.com\n");

    // one byte for new line
    commit_content.push(b'\n');
    // unknows bytes, read until end for msg
    commit_content.extend_from_slice(commit_msg.as_bytes());

    commit_content
}

fn write_commit_file(objects_path: &PathBuf, commit_content: Vec<u8>) -> io::Result<Vec<u8>> {
    let (commit_file_name, commit_file_hash) = get_hash_from_file(&commit_content);
    ioutils::save_file_hash(&commit_file_name, &objects_path, &commit_content)?;

    Ok(commit_file_hash)
}

fn fetch_parent_commit_hash(main_file: &Path) -> io::Result<Vec<u8>> {
    let parent_commit_hash = match main_file.exists() {
        true => fs::read(main_file)?,
        false => Vec::new(),
    };

    Ok(parent_commit_hash)
}

fn write_head_commit(
    parent_commit_exists: bool,
    main_file: &Path,
    commit_file_name: Vec<u8>,
) -> io::Result<()> {
    let mut f: fs::File;
    if parent_commit_exists {
        f = fs::OpenOptions::new().write(true).open(main_file)?;
    } else {
        fs::create_dir_all(main_file.parent().unwrap())?;
        f = fs::File::create(main_file).unwrap();
    }
    f.write_all(&commit_file_name)?;
    Ok(())
}

pub fn commit_rit(commit_msg: &str) {
    let objects_path = match ioutils::get_objects_path() {
        Ok(op) => op,
        Err(e) => {
            eprintln!("Error Getting 'objects' Path: {}", e);
            return;
        }
    };

    let main_file = Path::new("./.rit/refs/heads/main");
    let parent_commit_hash = match fetch_parent_commit_hash(main_file) {
        Ok(pa) => pa,
        Err(e) => {
            eprintln!("Error Fetching Parent Commit Hash: {}", e);
            return;
        }
    };

    let index_entries = match ioutils::read_index() {
        Ok((_, ie)) => ie,
        Err(e) => {
            eprintln!("Error Reading Index: {}", e);
            return;
        }
    };

    let tree_file_hash = match write_tree_file(&objects_path, index_entries) {
        Ok(p) => p,
        Err(e) => {
            match e.kind() {
                io::ErrorKind::AlreadyExists => println!("** Everything Up-to-date **"),
                _ => eprintln!("Error Writing Tree File: {}", e),
            }
            return;
        }
    };

    let commit_content = prepare_commit_content(&parent_commit_hash, tree_file_hash, commit_msg);

    let commit_file_hash = match write_commit_file(&objects_path, commit_content) {
        Ok(res) => res,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };

    if let Err(e) = write_head_commit(!parent_commit_hash.is_empty(), main_file, commit_file_hash) {
        eprintln!("Error Writing Head Commit: {}", e);
    }
}
