use sha1::{Digest, Sha1};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub fn add_rit(paths: Vec<&PathBuf>) {
    let rit_dir = Path::new(".rit");
    let parent_dir = rit_dir.join("objects");

    if !Path::exists(&parent_dir) {
        println!("no .rit found in curr dir");
        return;
    }

    for path in paths {
        if !Path::exists(&path) {
            println!("{path:?} does NOT exist; skipping...");
            continue;
        }
        let mut hasher = Sha1::new();

        if Path::is_file(&path) {
            let content = fs::read(path).unwrap();
            hasher.update(&content);

            let filename = format!("{:x}", hasher.finalize());
            let dirname = &filename[..3];
            let pathname = Path::join(&parent_dir, dirname);

            fs::create_dir(&pathname).unwrap();

            let final_path = Path::join(&pathname, filename);

            match fs::write(final_path, &content) {
                Ok(_) => println!("saved in objects"),
                Err(e) => println!("{e:?}"),
            }
        } else {
            println!("directory not supported now");
        }
    }
}
