use clap::{Arg, ArgAction, Command, command};

pub fn get_cli() -> Command {
    command!()
        .disable_help_subcommand(true)
        .disable_help_flag(true)
        .args_conflicts_with_subcommands(true)
        .arg(
            Arg::new("command")
                .help("The command to run")
                .trailing_var_arg(true)
                .num_args(0..)
                .allow_hyphen_values(true)
                .required_unless_present_any(["clear"])
                .conflicts_with_all(["shell", "login"]),
        )
        .arg(
            Arg::new("preserve")
                .short('e')
                .long("preserve-env")
                .action(ArgAction::SetTrue)
                .help("Preserve environment variables")
                .long_help("Preserves all environment variables. Potentially unsafe, use only when needed."),
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
                .help("The user to run as")
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
        .arg(Arg::new("shell").short('s').long("shell").num_args(1).default_value("PASSWD").value_name("shell").help("Creates a shell as root, preserving $HOME"))
        .arg(
            Arg::new("login")
                .short('l')
                .long("login")
                .help("Run the target user's shell, simulating a login")
                .conflicts_with("shell")
                .action(ArgAction::SetTrue),
        )
        .arg(Arg::new("help").long("help").short('h').help("Display this help output").action(ArgAction::SetTrue).exclusive(true))
}
