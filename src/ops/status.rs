use crate::utils::{
    hashutils::get_hash_from_file,
    ioutils::{get_objects_path, read_index, IndexEntry},
};
use std::{
    collections::HashMap,
    fs::{self, read, read_dir},
    io::ErrorKind,
    path::{Path, PathBuf},
};
use walkdir::{DirEntry, WalkDir};

const IGNORED_PATHS: &[&str] = &[".", ".ritignore"];

fn get_ignore_list() -> Vec<PathBuf> {
    let mut ignore_list: Vec<PathBuf> = vec![];

    // Add entries from .ritignore file
    match std::fs::read_to_string(".ritignore") {
        Ok(res) => {
            for line in res.lines() {
                ignore_list.push(PathBuf::from(line));
            }
        }
        Err(e) => eprintln!("{}", e),
    }

    ignore_list
}

fn should_ignore(e: &DirEntry, ignore_list: &Vec<PathBuf>) -> bool {
    if let Some(name) = e.file_name().to_str() {
        ignore_list
            .iter()
            .any(|term| name.contains(term.to_str().unwrap_or("")))
    } else {
        false
    }
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

fn should_ignore_or_hidden(entry: &DirEntry, ignore_list: &Vec<PathBuf>) -> bool {
    if let Some(path_str) = entry.path().to_str() {
        if IGNORED_PATHS.contains(&path_str) {
            return false;
        }
    }
    should_ignore(entry, ignore_list) || is_hidden(entry)
}

fn get_all_paths(ignore_list: Vec<PathBuf>) -> Vec<PathBuf> {
    let root_dir = Path::new(".");
    let mut paths = vec![];

    // search for files in "."
    if let Ok(entries) = read_dir(root_dir) {
        for e in entries.filter_map(|e| e.ok()) {
            if e.path().is_file() {
                paths.push(e.path());
            }
        }
    }

    // search all subdirs
    for entry in WalkDir::new(root_dir)
        .into_iter()
        .filter_entry(|e| !should_ignore_or_hidden(e, &ignore_list))
        // skip the non-permitted dirs
        .filter_map(|e| e.ok())
    {
        if !entry.path().is_dir() {
            paths.push(entry.into_path());
        }
    }

    paths
}

fn retrieve_committed_content() -> Result<HashMap<PathBuf, Vec<u8>>, std::io::Error> {
    let objects_path = get_objects_path()?;

    let main_file = Path::new("./.rit/refs/heads/main");
    let commit_hash = fs::read_to_string(main_file)?;

    let commit_dir = &commit_hash[..3];
    let commit_filename = &commit_hash[3..];

    let commit_path = Path::new(&objects_path)
        .join(commit_dir)
        .join(commit_filename);

    let commit_content = fs::read(commit_path)?;

    let mut tree_hash = String::new();
    for line in commit_content.split(|&b| b == b'\n') {
        if let Ok(line_str) = std::str::from_utf8(line) {
            if let Some(th) = line_str.split(' ').nth(1) {
                tree_hash = th.to_string();
                break;
            }
        }
    }
    if tree_hash.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "unable to find tree hash in commit file",
        ));
    }

    let tree_dir = &tree_hash[..3];
    let tree_filename = &tree_hash[3..];

    let tree_path = Path::new(&objects_path).join(tree_dir).join(tree_filename);

    let tree_content = fs::read(tree_path)?;

    let mut committed_content: HashMap<PathBuf, Vec<u8>> = HashMap::new();
    for line in tree_content.split(|&b| b == b'\n') {
        if let Ok(line_str) = std::str::from_utf8(line) {
            // part 1 permission, part 2 path, part 3 hash
            // for every line of the tree file
            let parts: Vec<&str> = line_str.split(' ').collect();

            if let (Some(h), Some(s)) = (parts.get(1), parts.get(2)) {
                committed_content.insert(PathBuf::from(*h), hex::decode(s).unwrap());
            }
        }
    }

    Ok(committed_content)
}

/// To check the status, we need to consider serveral factors
/// 1. Fetch all paths in current dir excluding .gitignore
/// 2. Check the INDEX to see what's been added
/// 3. Extract paths that are not part of INDEX
/// 4. Check if the files added have been changed (based on hash)
/// 5. Show the added paths as green, untracked paths as red, and modified as yellow
pub fn status_rit() {
    if !Path::new(".rit").exists() {
        eprintln!("rit has not been initialized in this dir!\n\nrun this command:\n> rit init");
        return;
    }

    let all_paths = get_all_paths(get_ignore_list());

    let committed_content;
    let mut check_commited = true;
    match retrieve_committed_content() {
        Ok(cc) => committed_content = cc,
        Err(e) => {
            eprintln!("Error reading committed content: {}", e);
            if e.kind() != ErrorKind::NotFound {
                return;
            } else {
                committed_content = HashMap::new();
                check_commited = false;
            }
        }
    }

    let mut untracked: Vec<PathBuf> = Vec::new();
    let mut modifed_unstaged: Vec<PathBuf> = Vec::new();
    let mut staged_uncommitted: Vec<PathBuf> = Vec::new();
    // let mut committed: Vec<PathBuf> = Vec::new();
    // let mut deleted = Vec::new();

    let mut index_entries: HashMap<PathBuf, IndexEntry> = HashMap::new();
    match read_index() {
        Ok((_, ies)) => ies.into_iter().for_each(|ie| {
            index_entries.insert(PathBuf::from(&ie.file_path), ie);
        }),
        Err(e) => {
            if e.kind() != ErrorKind::NotFound {
                eprintln!("{}", e);
                return;
            }
        }
    }

    all_paths.iter().for_each(|p| {
        let content = read(p).unwrap();
        let (_, hash_vec) = get_hash_from_file(&content);

        // 1. Check if committed, No action required for these
        if check_commited {
            if let Some(h) = committed_content.get(p) {
                println!("{:?}", *h);
                println!("{:?}", hash_vec);
                if *h == hash_vec {
                    // committed.push(p.to_path_buf());
                    return;
                }
            }
        }

        // 2 & 3. Check if staged modified or uncommitted
        if let Some(entry) = index_entries.get(p) {
            if hash_vec != entry.sha_hash {
                modifed_unstaged.push(p.to_path_buf())
            } else {
                staged_uncommitted.push(p.to_path_buf());
            }
            return;
        }

        // 4. Collect untracked
        untracked.push(p.to_path_buf());
    });

    println!("---------------------------");
    println!("\u{1b}[1;31mUntracked:\u{1b}[0m");
    println!("To add the file:\n>> rit add <PATH>...");
    for up in untracked {
        println!("\t\u{1b}[1;31m*\u{1b}[0m {}", up.display());
    }

    println!("---------------------------");
    println!("\u{1b}[1;32mTracked:\u{1b}[0m");
    for su in staged_uncommitted {
        println!("\t\u{1b}[1;32m$\u{1b}[0m {}", su.display());
    }

    println!("---------------------------");
    println!("\u{1b}[1;33mModified:\u{1b}[0m");
    for mp in modifed_unstaged {
        println!("\t\u{1b}[1;33m>\u{1b}[0m {}", mp.display());
    }
}
