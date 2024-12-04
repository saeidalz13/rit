use std::fs;

use crate::{
    models::errormodels::CliError,
    utils::{ioutils::get_config_path, terminalutils::print_success_msg},
};

pub fn rit_remote_set_url(remote_url: &String) -> Result<(), CliError> {
    let config_path = get_config_path()?;

    let config_content = fs::read_to_string(&config_path)?;
    let mut new_content = String::new();

    for line in config_content.lines() {
        if line.is_empty() {
            new_content.push('\n');
            continue;
        }

        match &line[..3] {
            "url" => {
                new_content.push_str("url=");
                new_content.push_str(&remote_url);
                new_content.push('\n');
            }
            _ => {
                new_content.push_str(line);
                new_content.push('\n');
            }
        }
    }

    fs::write(config_path, new_content)?;

    print_success_msg("remote url set!");
    Ok(())
}
