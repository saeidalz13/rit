use crate::{
    cli::commands,
    ops::{add::add_rit, commit::commit_rit, init::init_rit, status::status_rit},
};
use std::path::PathBuf;

pub fn exec_cli() {
    let matches = commands::get_commands().get_matches();

    match matches.subcommand() {
        Some(("add", sub_matches)) => {
            let paths = sub_matches
                .get_many::<PathBuf>("PATH")
                .into_iter()
                .flatten()
                .collect::<Vec<_>>();

            match add_rit(paths) {
                Ok(_) => println!("files added"),
                Err(e) => eprintln!("{}", e.to_string()),
            };
        }

        Some(("init", _)) => match init_rit() {
            Ok(_) => println!("rit initalized!"),
            Err(e) => eprintln!("{:?}", e.to_string()),
        },

        Some(("status", _)) => status_rit(),

        Some(("commit", sub_matches)) => {
            let commit_msg = sub_matches
                .get_one::<String>("message")
                .unwrap()
                .trim()
                .to_owned();

            commit_rit(&commit_msg);
        }

        _ => unreachable!(),
    }
}
