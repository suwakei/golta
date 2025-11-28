use clap::CommandFactory;
use clap_complete::{generate, Shell};
use std::io;

use crate::Cli;

pub fn run(shell: Shell, buf: &mut dyn io::Write) {
    let mut cmd = Cli::command();
    generate_completions(shell, &mut cmd, buf);
}

pub(crate) fn generate_completions(shell: Shell, cmd: &mut clap::Command, buf: &mut dyn io::Write) {
    let cmd_name = cmd.get_name().to_string();
    generate(shell, cmd, cmd_name, buf);
}
