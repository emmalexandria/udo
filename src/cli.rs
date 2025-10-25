use std::path::PathBuf;

use clap::{Arg, ArgAction, Command, command, value_parser};

pub fn get_cli() -> Command {
    command!()
        .disable_help_subcommand(true)
        .arg(
            Arg::new("command")
                .help("The command to run")
                .trailing_var_arg(true)
                .num_args(0..)
                .required_unless_present_any(["clear"]),
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
        .subcommand(
            command!("--install")
                .about("Creates config, PAM service, and sets correct permissions binary")
                .visible_short_flag_alias('I')
                .arg(Arg::new("pam").short('p').long("pam"))
                .arg(Arg::new("config").short('c').long("config")),
        )
        .subcommand(
            command!("--shell")
                .about("Runs a shell as the given user, optionally imitating a login")
                .visible_short_flag_alias('s')
                .arg(
                    Arg::new("imitate")
                        .help("Imitate a full login as the given user")
                        .long_help("Imitate a full login as the given user, loading their config, path, etc")
                        .short('i')
                        .long("imitate")
                    .action(ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("shell")
                        .short('s')
                        .long("s")
                        .help("Override the launched shell"),
                ),
        )
        .subcommand(
            command!("--config")
            .about("Manage your udo config")
            .visible_short_flag_alias('C')
            .arg(Arg::new("print").short('p').long("print").action(ArgAction::SetTrue))
            .arg(Arg::new("validate").short('v').long("validate").num_args(0..1).default_missing_value("/etc/udo/config.toml"))
        )
        .subcommand_negates_reqs(true)
}
