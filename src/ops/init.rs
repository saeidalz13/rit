use std::io::{self, Error, ErrorKind};
use std::{fs, path::Path};

pub fn init_rit() -> io::Result<()> {
    let parent_dir = Path::new(".rit");

    if parent_dir.exists() {
        return Err(Error::new(
            ErrorKind::AlreadyExists,
            "rit already initialized in this dir",
        ));
    };

    fs::create_dir_all(".rit/objects")?;
    fs::create_dir(".rit/hooks")?;
    fs::create_dir(".rit/info")?;
    fs::create_dir(".rit/logs")?;
    fs::create_dir(".rit/refs")?;
    fs::create_dir(".rit/rr-cache")
}
