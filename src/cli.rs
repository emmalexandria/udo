use clap::{Arg, ArgAction, Command, command};

pub fn get_cli() -> Command {
    command!()
        .disable_help_subcommand(true)
        .args_conflicts_with_subcommands(true)
        .arg(
            Arg::new("command")
                .help("The command to run")
                .trailing_var_arg(true)
                .num_args(0..)
                .allow_hyphen_values(true)
                .required_unless_present_any(["clear"]),
        )
        .arg(
            Arg::new("preserve")
                .short('e')
                .long("preserve-env")
                .action(ArgAction::SetTrue)
                .help("Preserve environment variables"),
        )
        .arg(
            Arg::new("nocheck")
                .help("Skips validating the permissions and owner of udo")
                .short('n')
                .long("nocheck")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("user")
                .short('u')
                .long("user")
                .num_args(1)
                .default_value("root"),
        )
        .arg(
            Arg::new("clear")
                .short('c')
                .long("clear")
                .action(ArgAction::SetTrue)
                .help("Clear the login cache"),
        )
        .arg(Arg::new("shell").short('s').long("shell").num_args(0..1))
        .arg(Arg::new("login").short('l').long("login"))
}
