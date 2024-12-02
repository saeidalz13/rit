use crate::utils::{hashutils::get_hash_from_file, ioutils};
use std::{
    fs, io,
    io::Write,
    path::{Path, PathBuf},
};

/// Tree file which contains:
/// - <MODE> <FILENAME> <HASH>
/// - where <MODE> is permission in octal. (100644 normal files) (100755 executables) (040000 subdirs), etc.
fn write_tree_file(objects_path: &PathBuf) -> io::Result<Vec<u8>> {
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
fn write_commit_file(
    objects_path: &PathBuf,
    parent_commit_hash: Vec<u8>,
    tree_file_hash: Vec<u8>,
    commit_msg: &str,
) -> io::Result<String> {
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

    let (commit_file_name, _) = get_hash_from_file(&commit_content);
    ioutils::save_file_hash(&commit_file_name, &objects_path, &commit_content)?;

    Ok(commit_file_name)
}

fn fetch_parent_commit_hash(main_file: &Path) -> io::Result<(Vec<u8>, bool)> {
    let mut parent_commit_hash: Vec<u8> = Vec::new();
    let mut main_file_exists = false;
    if main_file.exists() {
        parent_commit_hash = fs::read(main_file)?;
        main_file_exists = true;
    }

    Ok((parent_commit_hash, main_file_exists))
}

fn write_head_commit(
    main_file_exists: bool,
    main_file: &Path,
    commit_file_name: String,
) -> io::Result<bool> {
    let mut f: fs::File;
    if main_file_exists {
        f = fs::OpenOptions::new().write(true).open(main_file)?;
    } else {
        fs::create_dir_all(main_file.parent().unwrap())?;
        f = fs::File::create(main_file).unwrap();
    }
    f.write_all(commit_file_name.as_bytes())?;

    Ok(true)
}

pub fn commit_rit(commit_msg: &str) {
    let objects_path = ioutils::get_objects_path().unwrap();
    let main_file = Path::new("./.rit/refs/heads/main");
    let (parent_commit_hash, main_file_exists) = fetch_parent_commit_hash(main_file).unwrap();

    let tree_file_hash: Vec<u8>;
    match write_tree_file(&objects_path) {
        Ok(p) => tree_file_hash = p,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    }

    let commit_file_name = write_commit_file(
        &objects_path,
        parent_commit_hash,
        tree_file_hash,
        commit_msg,
    )
    .unwrap();

    write_head_commit(main_file_exists, main_file, commit_file_name).unwrap();
}
