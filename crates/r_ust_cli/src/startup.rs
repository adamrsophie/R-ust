use std::io::{self, Write};
use std::path::Path;

use super::{launcher, run_with_file, write_console, CliConfig, CliMode, OutputEvent};

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

fn argument_message(option: &str) -> String {
    let required = if option == "-e" {
        "a non-empty argument"
    } else {
        "a filename argument"
    };
    format!("ERROR: option '{option}' requires {required}")
}

pub(super) fn preflight(args: &[String]) -> Result<Option<CliMode>, StartupError> {
    let mut args = args.iter();
    let mut version = false;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => return Ok(Some(CliMode::Help)),
            "--version" => version = true,
            "--args" => break,
            "-e" => {
                let expression = args.next().ok_or(StartupError::Argument("-e"))?;
                launcher::encode_expression(expression).map_err(StartupError::Expression)?;
            }
            "-f" => {
                let file = args.next().ok_or(StartupError::Argument("-f"))?;
                match launcher::encode_expression(file) {
                    Ok(_) => {}
                    Err(launcher::ExpressionError::Invalid) => {
                        return Err(StartupError::Argument("-f"))
                    }
                    Err(error) => {
                        return Err(StartupError::Unsupported(format!(
                            "filename launcher conversion: {error}"
                        )))
                    }
                }
            }
            "--file" => {
                args.next().filter(|file| !file.is_empty()).ok_or_else(|| {
                    StartupError::Unsupported("--file requires a file path".into())
                })?;
            }
            "-" => {}
            _ if arg.starts_with('-') => {}
            _ => break,
        }
    }
    Ok(version.then_some(CliMode::Version))
}

/// Runs the main frontend with a complete invocation, including its program name.
pub fn run_main<I, S>(args: I) -> Result<i32, String>
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
