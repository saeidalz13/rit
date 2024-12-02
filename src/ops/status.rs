use crate::utils::{
    hashutils::get_hash_from_file,
    ioutils::{get_objects_path, read_index, IndexEntry},
};
use std::{
    collections::HashMap,
    fs::{self, read, read_dir},
    io,
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

fn get_head_commit_path(objects_path: &Path) -> io::Result<PathBuf> {
    let main_file = Path::new("./.rit/refs/heads/main");
    let commit_hash = fs::read_to_string(main_file)?;

    let commit_dir = &commit_hash[..3];
    let commit_filename = &commit_hash[3..];

    Ok(Path::new(&objects_path)
        .join(commit_dir)
        .join(commit_filename))
}

fn retrieve_committed_content() -> io::Result<HashMap<PathBuf, Vec<u8>>> {
    let objects_path = get_objects_path()?;
    let commit_path = get_head_commit_path(&objects_path)?;

    let commit_content = fs::read(commit_path)?;

    // hashed version of SHA256, hence 32 bytes
    let tree_hash = hex::encode(&commit_content[..32]);
    if tree_hash.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "unable to find tree hash in commit file",
        ));
    }

    let tree_path = Path::new(&objects_path)
        .join(&tree_hash[..3])
        .join(&tree_hash[3..]);
    let tree_content = fs::read(tree_path)?;

    let mut committed_content: HashMap<PathBuf, Vec<u8>> = HashMap::new();
    let mut pos: usize = 0;
    while pos < tree_content.len() - 1 {
        // let mode = &tree_content[pos..pos+4];
        let file_path_len_bytes: [u8; 4] = tree_content[pos + 4..pos + 8]
            .try_into()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let file_path_len = u32::from_be_bytes(file_path_len_bytes) as usize;
        let file_path = String::from_utf8_lossy(&tree_content[pos + 8..8 + pos + file_path_len]);
        pos = 8 + pos + file_path_len;

        let file_hash = &tree_content[pos..pos + 32];
        pos += 32;
        committed_content.insert(PathBuf::from(file_path.into_owned()), file_hash.to_owned());

        pos = (pos + 7) & !7;
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
            if e.kind() != io::ErrorKind::NotFound {
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
            if e.kind() != io::ErrorKind::NotFound {
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
                if *h == hash_vec {
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

    let mut is_everything_updated = true;

    if !untracked.is_empty() {
        is_everything_updated = false;
        println!("---------------------------");
        println!("\u{1b}[1;31mUntracked:\u{1b}[0m");
        println!("To add the file:\n>> rit add <PATH>...\n");
        for up in untracked {
            println!("\t\u{1b}[1;31m*\u{1b}[0m {}", up.display());
        }
    }

    if !staged_uncommitted.is_empty() {
        is_everything_updated = false;
        println!("---------------------------");
        println!("\u{1b}[1;32mTracked:\u{1b}[0m");
        for su in staged_uncommitted {
            println!("\t\u{1b}[1;32m$\u{1b}[0m {}", su.display());
        }
    }

    if !modifed_unstaged.is_empty() {
        is_everything_updated = false;
        println!("---------------------------");
        println!("\u{1b}[1;33mModified:\u{1b}[0m");
        for mp in modifed_unstaged {
            println!("\t\u{1b}[1;33m>\u{1b}[0m {}", mp.display());
        }
    }

    if is_everything_updated {
        println!("** Everything is up-to-date **")
    }
}
