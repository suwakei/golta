use clap::CommandFactory;
use clap_complete::Shell;
use std::io;

use crate::Cli;

pub fn run(shell: Shell, buf: &mut dyn io::Write) {
    let mut cmd = Cli::command();
    generate_completions(shell, &mut cmd, buf);
}

pub(crate) fn generate_completions(shell: Shell, cmd: &mut clap::Command, buf: &mut dyn io::Write) {
    let generator = ClapCompletionGenerator;
    generate_completions_with(shell, cmd, buf, &generator);
}

fn generate_completions_with(
    shell: Shell,
    cmd: &mut clap::Command,
    buf: &mut dyn io::Write,
    generator: &impl CompletionGenerator,
) {
    let cmd_name = cmd.get_name().to_string();
    generator.generate(shell, cmd, &cmd_name, buf);
}

trait CompletionGenerator {
    fn generate(
        &self,
        shell: Shell,
        cmd: &mut clap::Command,
        cmd_name: &str,
        buf: &mut dyn io::Write,
    );
}

struct ClapCompletionGenerator;

impl CompletionGenerator for ClapCompletionGenerator {
    fn generate(
        &self,
        shell: Shell,
        cmd: &mut clap::Command,
        cmd_name: &str,
        buf: &mut dyn io::Write,
    ) {
        clap_complete::generate(shell, cmd, cmd_name.to_string(), buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap_complete::Shell;
    use std::cell::RefCell;

    #[derive(Default)]
    struct MockGenerator {
        shell: RefCell<Option<Shell>>,
        cmd_name: RefCell<Option<String>>,
        invocations: RefCell<u32>,
    }

    impl CompletionGenerator for MockGenerator {
        fn generate(
            &self,
            shell: Shell,
            _cmd: &mut clap::Command,
            cmd_name: &str,
            buf: &mut dyn io::Write,
        ) {
            *self.invocations.borrow_mut() += 1;
            self.shell.replace(Some(shell));
            self.cmd_name.replace(Some(cmd_name.to_string()));
            let _ = buf.write_all(b"test-output");
        }
    }

    #[test]
    fn uses_command_name_and_shell_when_generating() {
        let mut cmd = clap::Command::new("golta-cli");
        let mut buf: Vec<u8> = Vec::new();
        let generator = MockGenerator::default();

        generate_completions_with(Shell::Bash, &mut cmd, &mut buf, &generator);

        assert_eq!(*generator.invocations.borrow(), 1);
        assert_eq!(generator.shell.borrow().unwrap(), Shell::Bash);
        assert_eq!(generator.cmd_name.borrow().as_deref(), Some("golta-cli"));
        assert_eq!(buf, b"test-output");
    }
}
