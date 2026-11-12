use std::io::{self, Write};

use super::{launcher, run, write_console, CliConfig, CliMode, OutputEvent};

#[derive(Debug)]
pub(super) enum StartupError {
    Argument(&'static str),
    Expression(launcher::ExpressionError),
    Fatal { message: String, stdout: Vec<u8> },
    Unsupported(String),
}

impl StartupError {
    pub(super) fn message(self) -> String {
        match self {
            Self::Argument(option) => argument_message(option),
            Self::Expression(error) => error.to_string(),
            Self::Fatal { message, .. } | Self::Unsupported(message) => message,
        }
    }
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
    let mut config = match CliConfig::parse_args(args) {
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
                StartupError::Fatal { message, stdout } => {
                    if !stdout.is_empty() {
                        let mut output = io::stdout().lock();
                        output
                            .write_all(&stdout)
                            .and_then(|_| output.flush())
                            .map_err(|error| error.to_string())?;
                    }
                    (OutputEvent::Stdout(format!("Fatal error: {message}\n")), 2)
                }
                StartupError::Unsupported(message) => {
                    (OutputEvent::Stderr(format!("{message}\n")), 2)
                }
            };
            write_console(event).map_err(|error| error.to_string())?;
            return Ok(status);
        }
    };
    config.command_args[0] = program;
    run(config)
}
