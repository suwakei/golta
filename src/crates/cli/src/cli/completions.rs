use clap::CommandFactory;
use clap_complete::{generate, Shell};
use std::io;

use crate::Cli;

pub fn run(shell: Shell) {
    let mut cmd = Cli::command();
    let cmd_name = cmd.get_name().to_string();
    generate(shell, &mut cmd, cmd_name, &mut io::stdout());
}
