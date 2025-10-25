use std::path::PathBuf;

use clap::{Arg, ArgAction, Command, command, value_parser};

pub fn get_cli() -> Command {
    command!()
        .arg(
            Arg::new("command")
                .help("The command to run")
                .trailing_var_arg(true)
                .num_args(0..)
                .required_unless_present("shell"),
        )
        .arg(
            Arg::new("shell")
                .help("Enter an elevated shell")
                .short('s')
                .long("shell")
                .action(ArgAction::SetTrue)
                .conflicts_with("command"),
        )
        .subcommand(
            command!("check")
                .about("Check if a given file is a valid udo configuration")
                .arg(
                    Arg::new("path")
                        .value_parser(value_parser!(PathBuf))
                        .num_args(1),
                ),
        )
}
