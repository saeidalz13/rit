use std::path::Path;
use walkdir::{DirEntry, WalkDir};

fn should_ignore(e: &DirEntry, ignore_list: &Vec<&str>) -> bool {
    if let Some(name) = e.file_name().to_str() {
        ignore_list.iter().any(|&term| name.contains(term))
    } else {
        false
    }
}

fn get_rit_paths() -> Vec<String> {
    let root_dir = Path::new(".");
    let mut paths = vec![];

    // TODO: get ignore_list from .ritignore
    let ignore_list = vec![".git", "targer", ".rit", "target"];

    for entry in WalkDir::new(root_dir)
        .into_iter()
        .filter_entry(|e| !should_ignore(e, &ignore_list))
    {
        match entry {
            Ok(e) => {
                if !e.path().is_dir() {
                    paths.push(e.path().display().to_string());
                }
            }
            Err(err) => eprintln!("{}", err),
        }
    }

    paths
}

pub fn status_rit() {
    get_rit_paths().into_iter().for_each(|p| {
        println!("{}", p.as_str());
    });
}
