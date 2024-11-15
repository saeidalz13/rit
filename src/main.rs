use rit::cli::commands;
use rit::ops::add::add_rit;
use rit::ops::init::init_rit;
use std::path::PathBuf;

fn main() {
    let matches = commands::get_commands().get_matches();

    match matches.subcommand() {
        Some(("add", sm)) => {
            let paths = sm
                .get_many::<PathBuf>("PATH")
                .into_iter()
                .flatten()
                .collect::<Vec<_>>();

            add_rit(paths);
        }
        Some(("init", _)) => init_rit(),
        _ => unreachable!(),
    }
}
