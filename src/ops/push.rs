use crate::models::errormodels::CliError;
use std::{fs, io, path::Path};

pub fn push_rit() -> Result<(), CliError> {
    let config_path = Path::new(".rit/config");
    if !config_path.exists() {
        return Err(CliError::from(io::Error::new(
            io::ErrorKind::NotFound,
            "config file does not exist.",
        )));
    }

    let config_content = fs::read_to_string(config_path)?;
    let mut remote_url = String::new();
    for line in config_content.lines().into_iter() {
        if &line[..3] == "url" {
            remote_url.push_str(&line[4..]);
            break;
        }
    }
    if remote_url.is_empty() {
        return Err(CliError::GeneralError("remote url does not exist"));
    }

    println!("remote url: {}", remote_url);

    println!("Code pushed!");
    Ok(())
}
