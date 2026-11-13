use std::io::{self, Write};
use std::path::Path;

use super::{launcher, run_with_file, write_console, CliConfig, OutputEvent};

pub(super) enum Event<'a> {
    File(&'a Path),
    Stdout(&'a [u8]),
}

#[derive(Debug)]
pub(super) enum StartupError {
    Argument(&'static str),
    Expression(launcher::ExpressionError),
    Fatal(String),
    Unsupported(String),
    Output(io::Error),
}

impl StartupError {
    pub(super) fn message(self) -> String {
        match self {
            Self::Argument(option) => argument_message(option),
            Self::Expression(launcher::ExpressionError::Invalid) => argument_message("-e"),
            Self::Expression(error) => error.to_string(),
            Self::Fatal(message) | Self::Unsupported(message) => message,
            Self::Output(error) => error.to_string(),
        }
    }
}

pub(super) fn file_error(path: &Path, error: &io::Error) -> String {
    let message = error.to_string();
    let message = message.split(" (os error").next().unwrap_or(&message);
    format!("cannot open file '{}': {message}", path.display())
}

pub(super) fn append_unknown_option(stdout: &mut Vec<u8>, option: &str) {
    append_message(
        stdout,
        &format!("WARNING: unknown option '{option}'\n"),
        1151,
    );
}

pub(super) fn append_message(stdout: &mut Vec<u8>, message: &str, capacity: usize) {
    let bytes = message.as_bytes();
    stdout.extend_from_slice(&bytes[..bytes.len().min(capacity)]);
    stdout.push(b'\n');
}

#[derive(Clone, Copy)]
pub(super) enum Frontend {
    Main,
    Rscript,
}

pub(super) fn is_backend_option(option: &str) -> bool {
    matches!(
        option,
        "--version"
            | "--args"
            | "--save"
            | "--no-save"
            | "--restore"
            | "--no-restore"
            | "--no-restore-data"
            | "--no-restore-history"
            | "--silent"
            | "--quiet"
            | "-q"
            | "--vanilla"
            | "--no-environ"
            | "--verbose"
            | "--no-echo"
            | "--slave"
            | "-s"
            | "--no-site-file"
            | "--no-init-file"
            | "--no-readline"
            | "--interactive"
            | "-f"
            | "-e"
            | "-save"
            | "-nosave"
            | "-restore"
            | "-norestore"
            | "-noreadline"
            | "-quiet"
            | "-nsize"
            | "-vsize"
            | "-V"
            | "-n"
            | "-v"
    ) || [
        "--encoding",
        "--min-nsize",
        "--min-vsize",
        "--max-nsize",
        "--max-vsize",
        "--max-ppsize",
        "--max-connections",
        "--file=",
        "--gui",
        "-g",
        "-psn",
    ]
    .iter()
    .any(|prefix| option.starts_with(prefix))
}

fn argument_message(option: &str) -> String {
    let required = if option == "-e" {
        "a non-empty argument"
    } else {
        "a filename argument"
    };
    format!("ERROR: option '{option}' requires {required}")
}

/// Runs the main frontend with a complete invocation, including its program name.
pub fn run_main<I, S>(args: I) -> Result<i32, String>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    run_invocation(args, Frontend::Main)
}

pub(super) fn run_invocation<I, S>(args: I, frontend: Frontend) -> Result<i32, String>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut args = args.into_iter().map(Into::into);
    let program = args.next().unwrap_or_else(|| "r_ust_cli".into());
    let mut opened = Vec::new();
    let mut config = match CliConfig::parse_args(args, |event| {
        match event {
            Event::File(path) => {
                opened.push(
                    std::fs::File::open(path)
                        .map_err(|error| StartupError::Fatal(file_error(path, &error)))?,
                );
            }
            Event::Stdout(bytes) => {
                let mut stdout = io::stdout().lock();
                stdout
                    .write_all(bytes)
                    .and_then(|_| stdout.flush())
                    .map_err(StartupError::Output)?;
            }
        }
        Ok(())
    }) {
        Ok(config) => config,
        Err(error) => {
            match (frontend, &error) {
                (
                    Frontend::Rscript,
                    StartupError::Argument(_)
                    | StartupError::Expression(_)
                    | StartupError::Unsupported(_),
                ) => return Err(error.message()),
                (Frontend::Rscript, StartupError::Fatal(_) | StartupError::Output(_))
                | (
                    Frontend::Main,
                    StartupError::Argument(_)
                    | StartupError::Expression(_)
                    | StartupError::Fatal(_)
                    | StartupError::Unsupported(_)
                    | StartupError::Output(_),
                ) => {}
            }
            let (event, status) = match error {
                StartupError::Argument(option) => (
                    OutputEvent::Stderr(format!("{}\n", argument_message(option))),
                    1,
                ),
                StartupError::Expression(error) => {
                    let (message, status) = match error {
                        launcher::ExpressionError::Invalid => (argument_message("-e"), 1),
                        other => (other.to_string(), 2),
                    };
                    (OutputEvent::Stderr(format!("{message}\n")), status)
                }
                StartupError::Fatal(message) => {
                    (OutputEvent::Stdout(format!("Fatal error: {message}\n")), 2)
                }
                StartupError::Unsupported(message) => {
                    (OutputEvent::Stderr(format!("{message}\n")), 2)
                }
                StartupError::Output(error) => return Err(error.to_string()),
            };
            write_console(event).map_err(|error| error.to_string())?;
            return Ok(status);
        }
    };
    config.command_args[0] = program;
    config.startup_stdout.clear();
    run_with_file(config, opened.pop())
}
