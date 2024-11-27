use std::{
    io::ErrorKind,
    path::{Path, PathBuf},
};
use walkdir::{DirEntry, WalkDir};

use crate::utils::ioutils::read_index;

fn get_ignore_list() -> Vec<PathBuf> {
    let mut ignore_list = vec![PathBuf::from(".rit"), PathBuf::from(".git")];

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

fn get_rit_paths(ignore_list: Vec<PathBuf>) -> Vec<PathBuf> {
    let root_dir = Path::new(".");
    let mut paths = vec![];

    for entry in WalkDir::new(root_dir)
        .into_iter()
        .filter_entry(|e| !should_ignore(e, &ignore_list))
    {
        match entry {
            Ok(e) => {
                if !e.path().is_dir() {
                    paths.push(e.into_path());
                }
            }
            Err(err) => eprintln!("{}", err),
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

    let mut untracked_paths = vec![];
    let mut tracked_paths = vec![];
    // let mut modified_paths = vec![];

    match read_index() {
        Ok((ih, ies)) => {
            println!("{:?}", ih);
            // iter() mean immutable borrow of the variables
            for ie in ies.iter() {
                for rp in rit_paths.clone().into_iter() {
                    if rp.display().to_string() == ie.file_path {
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

    println!("\u{1b}[1;31mUntracked:\u{1b}[0m");
    for up in untracked_paths {
        println!("\t\u{1b}[1;31m*\u{1b}[0m {}", up.display());
    }
}
