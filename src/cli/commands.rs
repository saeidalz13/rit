use clap::{arg, value_parser, Command};
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
                .arg(arg!(<PATH>..."paths to add").value_parser(value_parser!(PathBuf))),
        )
        // init command
        .subcommand(Command::new("init").about("initialize a repo"))
    // ...
}
