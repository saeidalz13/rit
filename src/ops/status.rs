use crate::utils::ioutils::read_index;
use std::{
    fs::read_dir,
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

fn get_rit_paths(ignore_list: Vec<PathBuf>) -> Vec<PathBuf> {
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

/// To check the status, we need to consider serveral factors
/// 1. Fetch all paths in current dir excluding .gitignore
/// 2. Check the INDEX to see what's been added
/// 3. Extract paths that are not part of INDEX
/// 4. Check if the files added have been changed (based on hash)
/// 5. Show the added paths as green, untracked paths as red, and modified as yellow
pub fn status_rit() {
    let rit_paths = get_rit_paths(get_ignore_list());

    let mut untracked_paths: Vec<PathBuf> = vec![];
    let mut tracked_paths: Vec<PathBuf> = vec![];
    // let mut modified_paths = vec![];

    match read_index() {
        Ok((ih, ies)) => {
            println!("{:?}", ih);

            // iter() mean immutable borrow of the variables
            for ie in ies.iter() {
                for rp in rit_paths.clone().into_iter() {
                    if rp.to_str().unwrap() == ie.file_path {
                        tracked_paths.push(rp);
                    } else {
                        untracked_paths.push(rp);
                    }
                }
            }
        }
        Err(e) => {
            if e.kind() == ErrorKind::NotFound {
                for rp in rit_paths.into_iter() {
                    untracked_paths.push(rp);
                }
            } else {
                eprintln!("{}", e);
                return;
            }
        }
    }

    println!("---------------------------");
    println!("\u{1b}[1;31mUntracked:\u{1b}[0m");
    for up in untracked_paths {
        println!("\t\u{1b}[1;31m*\u{1b}[0m {}", up.display());
    }

    println!("---------------------------");
    println!("\u{1b}[1;32mTracked:\u{1b}[0m");
    for tp in tracked_paths {
        println!("\t\u{1b}[1;32m$\u{1b}[0m {}", tp.display());
    }
}
