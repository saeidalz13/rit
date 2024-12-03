use clap::{arg, value_parser, Arg, ArgAction, Command};
use std::path::PathBuf;

pub fn get_commands() -> Command {
    Command::new("rit")
        .version("1.0.0")
        .about("git written in Rust")
        .subcommand_required(true)
        .arg_required_else_help(true)
        // add command
        .subcommand(
            Command::new("add")
                .about("add files to rit memory")
                .arg_required_else_help(true)
                .arg(
                    arg!(<PATH>..."paths to add")
                        .value_parser(value_parser!(PathBuf))
                        .required(false),
                )
                .arg(
                    Arg::new("all")
                        .short('a')
                        .long("all")
                        .help("Add all files to rit memory")
                        .action(ArgAction::SetTrue),
                    // implicitely set to false
                    // .required(false)
                ),
        )
        // init command
        .subcommand(Command::new("init").about("initialize a repo"))
        // status command
        .subcommand(Command::new("status").about("checks the status of rit dir"))
        // commit command
        .subcommand(
            Command::new("commit")
                .about("commits the staged files")
                .arg_required_else_help(true)
                .arg(
                    arg!(-m --message <COMMIT_MSG> "The commit message")
                        .value_parser(value_parser!(String)),
                ),
        )
        .subcommand(Command::new("push").about("upload the current commit to remote url"))
    // ...
}
