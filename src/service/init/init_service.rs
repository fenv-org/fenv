use crate::{
    args::FenvInitArgs,
    context::FenvContext,
    debug,
    sdk_service::sdk_service::SdkService,
    service::{completions::completions_service::FenvCompletionsService, service::Service},
    spawn_and_capture,
    util::io::ConsoleOutput,
};
use anyhow::{anyhow, bail, Context as _, Ok, Result};
use clap::ValueEnum;
use clap_complete::Shell;
use indoc::writedoc;
use lazy_static::lazy_static;
use nix::unistd::getppid;
use regex::Regex;
use std::{include_str, io::Write, process::Command};

pub struct FenvInitService {
    pub args: FenvInitArgs,
}

impl FenvInitService {
    pub fn new(args: FenvInitArgs) -> FenvInitService {
        FenvInitService { args }
    }

    fn execute_detect_shell<'a>(
        &self,
        context: &impl FenvContext,
        stdout: &mut impl Write,
    ) -> Result<()> {
        let shell = detect_shell(context).context("Failed to detect the interactive shell")?;
        writeln!(stdout, "FENV_SHELL_DETECT={}", shell)?;
        Ok(())
    }

    fn show_help<'a>(&self, context: &impl FenvContext, stdout: &mut impl Write) -> Result<()> {
        let shell = match &self.args.shell {
            Some(shell) => String::from(shell),
            None => detect_shell(context).context("Failed to detect the current shell")?,
        };

        match &shell[..] {
            "fish" => writedoc!(stdout, "{}", include_str!("fish/help.txt"))?,
            "bash" => writedoc!(stdout, "{}", include_str!("bash/help.txt"))?,
            "zsh" => writedoc!(stdout, "{}", include_str!("zsh/help.txt"))?,
            "ksh" => writedoc!(stdout, "{}", include_str!("ksh/help.txt"))?,
            _ => bail!("Unsupported shell: {shell}"),
        }
        writedoc!(stdout, "{}", include_str!("common/help_footer.txt"))?;
        Ok(())
    }

    fn print_path<'a>(
        &self,
        context: &impl FenvContext,
        detected_shell: &str,
        stdout: &mut impl Write,
    ) -> Result<()> {
        match &detected_shell[..] {
            "fish" => writedoc!(
                stdout,
                "{}",
                include_str!("fish/path_template.txt")
                    .replace("%FENV_ROOT%", &context.fenv_root().to_string()),
            ),
            _ => writedoc!(
                stdout,
                "{}",
                include_str!("common/path_template.txt")
                    .replace("%FENV_ROOT%", &context.fenv_root().to_string()),
            ),
        }
        .map_err(|e| anyhow!(e))
    }

    fn print_completions(&self, detected_shell: &str, stdout: &mut impl Write) -> Result<()> {
        let shell = Shell::from_str(detected_shell, true).map_err(|e| anyhow::anyhow!(e))?;
        let completions_commands = FenvCompletionsService::completions_commands(&shell);
        write!(stdout, "{}", completions_commands).map_err(|e| anyhow::anyhow!(e))
    }
}

impl<OUT, ERR> Service<OUT, ERR> for FenvInitService
where
    OUT: std::io::Write,
    ERR: std::io::Write,
{
    fn execute(
        &self,
        context: &impl FenvContext,
        _: &impl SdkService,
        output: &mut dyn ConsoleOutput<OUT, ERR>,
    ) -> anyhow::Result<()> {
        if self.args.detect_shell {
            return self.execute_detect_shell(context, output.stdout());
        }

        match self.args.path_mode {
            Some(_) => {
                let shell = match &self.args.shell {
                    Some(shell) => String::from(shell),
                    None => detect_shell(context).context("Failed to detect the current shell")?,
                };
                self.print_path(context, &shell, output.stdout())?;
                match &shell[..] {
                    "fish" | "bash" => self.print_completions(&shell, output.stdout()),
                    "zsh" => {
                        write!(output.stdout(), "{}", include_str!("zsh/path_footer.txt"))?;
                        Ok(())
                    }
                    _ => Ok(()),
                }
            }
            None => self.show_help(context, output.stdout()),
        }
    }
}

fn detect_shell<'a>(context: &impl FenvContext) -> Result<String> {
    // With `ps -o 'args='`,
    // captures the command line arguments which launched the shell.
    let ppid = getppid().as_raw();
    let mut command = Command::new("bash");
    let ps_output = spawn_and_capture!(
        command
            .arg("-c")
            .arg(format!("ps -p {ppid} -o 'args=' 2>/dev/null || true")),
        "detect_shell",
        "Failed to acquire the interactive shell name",
    );
    debug!("detect_shell(): stdout:\n`{ps_output}`");
    let executable_path = extract_shell_executable(context, &ps_output.trim_end());
    debug!("detect_shell(): executable_path=`{executable_path}`");
    Ok(extract_shell_name_from_executable_path(&executable_path).unwrap_or(executable_path))
}

/// Tries to extract a shell executable from the given `ps_output`.
///
/// If failed, fallback the `$SHELL` environment variable.
fn extract_shell_executable<'a>(context: &impl FenvContext, ps_output: &str) -> String {
    lazy_static! {
        static ref EXECUTABLE_PATTERN: Regex = Regex::new(r"^\s*\-*(\S+)(?:\s.*)?\s*$").unwrap();
    }
    match EXECUTABLE_PATTERN.captures(ps_output) {
        Some(capture) => String::from(capture.get(1).map(|s| s.as_str()).unwrap()),
        None => context.default_shell().to_owned(),
    }
}

fn extract_shell_name_from_executable_path(executable: &str) -> Option<String> {
    lazy_static! {
        static ref SHELL_NAME_PATTERN: Regex = Regex::new(r"^(?:.*/)([^/-]+)(?:-.*)?$").unwrap();
    }
    SHELL_NAME_PATTERN
        .captures(executable)
        .map(|capture| String::from(capture.get(1).map(|s| s.as_str()).unwrap()))
}

#[cfg(test)]
mod tests {
    use crate::{
        context::RealFenvContext,
        sdk_service::sdk_service::RealSdkService,
        service::{
            completions::completions_service::FenvCompletionsService, macros::test_with_context,
        },
        try_run,
        util::io::BufferedOutput,
    };
    use clap_complete::Shell;
    use indoc::indoc;

    fn new_context() -> RealFenvContext {
        RealFenvContext::new(
            "/home/user/.fenv",
            "/home/user/workspace",
            "/home/user",
            "/bin/fish",
            "/home/user/.pub-cache",
        )
    }

    #[test]
    fn test_fish_show_help() {
        test_with_context(|context, output| {
            // setup
            let sdk_service = RealSdkService::new();

            // execution
            try_run(
                &["fenv", "init", "--shell", "fish"],
                context,
                &sdk_service,
                output,
            )
            .unwrap();

            // validation
            assert_eq!(
                output.stdout_to_string(),
                indoc! {"
                    # Add fenv executable to PATH by running
                    # the following interactively:

                    set -Ux FENV_ROOT $HOME/.fenv
                    fish_add_path $FENV_ROOT/bin

                    # Load fenv automatically by appending
                    # the following to ~/.config/fish/conf.d/fenv.fish:

                    fenv init - | source

                    # Restart your shell for the changes to take effect:

                    exec $SHELL -l

                    "
                }
            )
        })
    }

    #[test]
    fn test_bash_show_help() {
        test_with_context(|context, output| {
            // setup
            let sdk_service = RealSdkService::new();

            // execution
            try_run(
                &["fenv", "init", "--shell", "bash"],
                context,
                &sdk_service,
                output,
            )
            .unwrap();

            // validation
            assert_eq!(
                output.stdout_to_string(),
                indoc! {r#"
                    # Load fenv automatically by appending the following to
                    # ~/.bash_profile if it exists, otherwise ~/.profile (for login shells)
                    # and ~/.bashrc (for interactive shells) :

                    export FENV_ROOT="$HOME/.fenv"
                    command -v fenv >/dev/null || export PATH="$FENV_ROOT/bin:$PATH"
                    eval "$(fenv init -)"

                    # Restart your shell for the changes to take effect:

                    exec $SHELL -l

                    "#
                }
            )
        })
    }

    #[test]
    fn test_zsh_show_help() {
        test_with_context(|context, output| {
            // setup
            let sdk_service = RealSdkService::new();

            // execution
            try_run(
                &["fenv", "init", "--shell", "zsh"],
                context,
                &sdk_service,
                output,
            )
            .unwrap();

            // validation
            assert_eq!(
                output.stdout_to_string(),
                indoc! {r#"
                    # Load fenv automatically by appending the following to
                    # ~/.zprofile (for login shells)
                    # and ~/.zshrc (for interactive shells) :

                    export FENV_ROOT="$HOME/.fenv"
                    command -v fenv >/dev/null || export PATH="$FENV_ROOT/bin:$PATH"
                    eval "$(fenv init -)"

                    # Restart your shell for the changes to take effect:

                    exec $SHELL -l

                    "#
                }
            )
        })
    }

    #[test]
    fn test_ksh_show_help() {
        test_with_context(|context, output| {
            // setup
            let sdk_service = RealSdkService::new();

            // execution
            try_run(
                &["fenv", "init", "--shell", "ksh"],
                context,
                &sdk_service,
                output,
            )
            .unwrap();

            // validation
            assert_eq!(
                output.stdout_to_string(),
                indoc! {r#"
                    # Load fenv automatically by appending the following to
                    # ~/.profile :

                    export FENV_ROOT="$HOME/.fenv"
                    command -v fenv >/dev/null || export PATH="$FENV_ROOT/bin:$PATH"
                    eval "$(fenv init -)"

                    # Restart your shell for the changes to take effect:

                    exec $SHELL -l

                    "#
                }
            )
        })
    }

    #[test]
    fn test_fish_path_help() {
        // setup
        let context = new_context();
        let mut output = BufferedOutput::new();
        let sdk_service = RealSdkService::new();

        // execution
        try_run(
            &["fenv", "init", "-", "--shell", "fish"],
            &context,
            &sdk_service,
            &mut output,
        )
        .unwrap();

        // validation
        assert_eq!(
            output.stdout_to_string(),
            indoc! {r#"
                    while set fenv_index (contains -i -- "/home/user/.fenv/shims" $PATH)
                    set -eg PATH[$fenv_index]; end; set -e fenv_index
                    set -gx PATH '/home/user/.fenv/shims' $PATH
                    %COMPLETIONS%"#,
            }
            .replace(
                "%COMPLETIONS%",
                &FenvCompletionsService::completions_commands(&Shell::Fish)
            )
        )
    }

    #[test]
    fn test_bash_path_help() {
        // setup
        let context = new_context();
        let mut output = BufferedOutput::new();
        let sdk_service = RealSdkService::new();

        // execution
        try_run(
            &["fenv", "init", "-", "--shell", "bash"],
            &context,
            &sdk_service,
            &mut output,
        )
        .unwrap();

        // validation
        assert_eq!(
                output.stdout_to_string(),
                indoc! {r#"
                    PATH="$(bash --norc -ec 'IFS=:; paths=($PATH);
                    for i in ${!paths[@]}; do
                    if [[ ${paths[i]} == "''/home/user/.fenv/shims''" ]]; then unset '\''paths[i]'\'';
                    fi; done;
                    echo "${paths[*]}"')"
                    export PATH="/home/user/.fenv/shims:${PATH}"
                    %COMPLETIONS%"#
                }
                .replace(
                    "%COMPLETIONS%",
                    &FenvCompletionsService::completions_commands(&Shell::Bash)
                )
            )
    }

    #[test]
    fn test_zsh_path_help() {
        // setup
        let context = new_context();
        let mut output = BufferedOutput::new();
        let sdk_service = RealSdkService::new();

        // execution
        try_run(
            &["fenv", "init", "-", "--shell", "zsh"],
            &context,
            &sdk_service,
            &mut output,
        )
        .unwrap();

        // validation
        assert_eq!(
            output.stdout_to_string(),
            indoc! {r#"
                PATH="$(bash --norc -ec 'IFS=:; paths=($PATH);
                for i in ${!paths[@]}; do
                if [[ ${paths[i]} == "''/home/user/.fenv/shims''" ]]; then unset '\''paths[i]'\'';
                fi; done;
                echo "${paths[*]}"')"
                export PATH="/home/user/.fenv/shims:${PATH}"
                if [[ -z "$(command -v compdef || true)" ]]; then
                  autoload -Uz compinit && compinit
                fi
                source <(fenv completions zsh)
                "#
            }
        )
    }

    #[test]
    fn test_ksh_path_help() {
        // setup
        let context = new_context();
        let mut output = BufferedOutput::new();
        let sdk_service = RealSdkService::new();

        // execution
        try_run(
            &["fenv", "init", "-", "--shell", "ksh"],
            &context,
            &sdk_service,
            &mut output,
        )
        .unwrap();

        // validation
        assert_eq!(
            output.stdout_to_string(),
            indoc! {r#"
                PATH="$(bash --norc -ec 'IFS=:; paths=($PATH);
                for i in ${!paths[@]}; do
                if [[ ${paths[i]} == "''/home/user/.fenv/shims''" ]]; then unset '\''paths[i]'\'';
                fi; done;
                echo "${paths[*]}"')"
                export PATH="/home/user/.fenv/shims:${PATH}"
                "#
            }
        )
    }
}
