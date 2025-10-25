use std::path::PathBuf;

use clap::{Arg, ArgAction, Command, command, value_parser};

pub fn get_cli() -> Command {
    command!()
        .arg(
            Arg::new("command")
                .help("The command to run")
                .trailing_var_arg(true)
                .num_args(0..)
                .required_unless_present_any(["shell", "interactive", "clear"]),
        )
        .arg(
            Arg::new("shell")
                .short('s')
                .long("shell")
                .action(ArgAction::SetTrue)
                .help("Enter an elevated shell"),
        )
        .arg(
            Arg::new("interactive")
                .short('i')
                .long("interactive")
                .action(ArgAction::SetTrue)
                .help("Simulate a full root login"),
        )
        .arg(
            Arg::new("clear")
                .short('c')
                .long("clear")
                .action(ArgAction::SetTrue)
                .help("Clear the login cache"),
        )
}
