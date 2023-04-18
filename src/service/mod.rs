pub mod service;

pub mod init {
    pub mod init_service;
}

pub mod install {
    mod flutter_command;
    mod git_command;
    mod install_sdk;
    pub mod install_service;
    mod list_remote_sdk;
}

pub mod macros {
    #[macro_export(local_inner_macros)]
    macro_rules! spawn_and_wait {
        ($expr: expr, $fn_name: expr, $($arg:tt)+) => {{
            let command = $expr;
            log::info!(
                "{}(): command: program={:?}: args={:?}",
                $fn_name,
                command.get_program(),
                command.get_args()
            );
            let child = &mut command.spawn().with_context(|| std::format!($($arg)+))?;
            let exit_status = &mut child.wait().with_context(|| std::format!($($arg)+))?;
            if !exit_status.success() {
                let message = std::format!($($arg)+);
                anyhow::bail!(
                    "{message}: OS state code - {code}",
                    code = exit_status.code().unwrap()
                )
            }
        }};
    }

    #[macro_export(local_inner_macros)]
    macro_rules! spawn_and_capture {
        ($expr: expr, $fn_name: expr, $($arg:tt)+) => {{
            let command = $expr;
            log::info!(
                "{}(): command: program={:?}: args={:?}",
                $fn_name,
                command.get_program(),
                command.get_args()
            );
            let output = command.output().with_context(|| std::format!($($arg)+))?;
            if !output.status.success() {
                log::debug!(
                    "{}(): stderr:\n{}",
                    $fn_name,
                    String::from_utf8(output.stderr)?
                );
                let message = std::format!($($arg)+);
                anyhow::bail!(
                    "{message}: OS state code - {code}",
                    code = output.status.code().unwrap()
                )
            }
            let stdout_output = String::from_utf8(output.stdout)?;
            stdout_output
        }};
    }
}
