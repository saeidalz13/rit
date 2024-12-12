use std::io;
use std::io::Write;
use std::{fs, path::Path};

use crate::models::errormodels::CliError;

fn create_config() -> Result<(), CliError> {
    let mut f = fs::File::create_new(".rit/config")?;

    // [core] copied from git config
    // TODO: look into it to see what each line is
    f.write(
        b"[core]
    bare=false
    repositoryformatversion=0
    filemode=true
    ignorecase=true
    precomposeunicode=true
    logallrefupdates=true

    [remote_origin]
    url=
    fetch=
    ",
    )?;

    Ok(())
}

pub fn init_rit() -> Result<(), CliError> {
    let parent_dir = Path::new(".rit");

    if parent_dir.exists() {
        return Err(CliError::from(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "rit already initialized in this dir",
        )));
    };

    fs::create_dir_all(".rit/objects")?;
    fs::create_dir(".rit/hooks")?;
    fs::create_dir(".rit/info")?;
    fs::create_dir(".rit/logs")?;
    fs::create_dir(".rit/refs")?;
    fs::create_dir(".rit/rr-cache")?;
    create_config()?;

    Ok(())
}
