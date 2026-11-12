use std::path::PathBuf;

use super::startup::StartupError;
use super::{launcher, startup};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CliMode {
    Interactive,
    Stdin,
    File(PathBuf),
    Expression(String),
    Version,
    Help,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CliConfig {
    pub mode: CliMode,
    pub quiet: bool,
    pub no_echo: bool,
    pub vanilla: bool,
    pub no_save: bool,
    /// R-visible invocation, including the program name and an optional --args separator.
    pub command_args: Vec<String>,
    /// Raw stdout emitted before the session starts; warning truncation may split UTF-8.
    pub startup_stdout: Vec<u8>,
}

impl CliConfig {
    pub fn from_args<I, S>(args: I) -> Result<Self, String>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self::parse_args(args, |_| Ok(())).map_err(StartupError::message)
    }

    pub(super) fn parse_args<I, S, F>(args: I, mut on_startup: F) -> Result<Self, StartupError>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
        F: FnMut(startup::Event<'_>) -> Result<(), StartupError>,
    {
        let mut quiet = false;
        let mut no_echo = false;
        let mut vanilla = false;
        let mut no_save = false;
        let mut mode = CliMode::Interactive;
        let mut startup_stdout = Vec::new();
        let mut expressions = String::new();
        let mut command_args = vec!["r_ust_cli".to_string()];
        command_args.extend(args.into_iter().map(Into::into));
        if let Some(mode) = startup::preflight(&command_args[1..])? {
            return Ok(Self {
                mode,
                command_args,
                ..Self::default()
            });
        }
        let mut args = command_args.clone().into_iter().skip(1).peekable();

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--quiet" | "--silent" | "-q" => quiet = true,
                "--no-echo" | "--slave" | "-s" => {
                    quiet = true;
                    no_echo = true;
                    no_save = true;
                }
                "--vanilla" => {
                    vanilla = true;
                    no_save = true;
                }
                "--no-save" => no_save = true,
                "--version" | "version" => mode = CliMode::Version,
                "--help" | "-h" => mode = CliMode::Help,
                "-" => {
                    if !expressions.is_empty() {
                        return Err(StartupError::Unsupported(
                            "cannot use stdin and -e together".into(),
                        ));
                    }
                    mode = CliMode::Stdin;
                }
                "-e" => {
                    let expression = args
                        .next()
                        .filter(|expression| !expression.is_empty())
                        .ok_or(StartupError::Argument("-e"))?;
                    let encoded = launcher::encode_expression(&expression)
                        .map_err(StartupError::Expression)?;
                    let index = command_args.len() - args.len() - 1;
                    command_args[index] = encoded.clone();
                    let previous = startup_stdout.len();
                    launcher::append_expression(&mut expressions, &mut startup_stdout, &encoded);
                    if startup_stdout.len() != previous {
                        on_startup(startup::Event::Stdout(&startup_stdout[previous..]))?;
                    }
                    if !matches!(mode, CliMode::File(_)) {
                        mode = CliMode::Stdin;
                    }
                }
                _ if arg == "--file" || arg == "-f" || arg.starts_with("--file=") => {
                    let file = arg
                        .strip_prefix("--file=")
                        .map(str::to_string)
                        .or_else(|| args.next())
                        .ok_or_else(|| {
                            if arg == "-f" {
                                StartupError::Argument("-f")
                            } else {
                                StartupError::Unsupported(format!("{arg} requires a file path"))
                            }
                        })?;
                    if file == "-" {
                        if !matches!(mode, CliMode::File(_)) {
                            mode = CliMode::Stdin;
                        }
                    } else {
                        let path = PathBuf::from(file);
                        on_startup(startup::Event::File(&path))?;
                        mode = CliMode::File(path);
                    }
                }
                "--args" => break,
                _ if arg.starts_with('-') && startup::is_backend_option(&arg) => {
                    return Err(StartupError::Unsupported(format!(
                        "unsupported CLI option {arg}"
                    )));
                }
                _ if arg.starts_with('-') => {
                    let previous = startup_stdout.len();
                    startup::append_unknown_option(&mut startup_stdout, &arg);
                    on_startup(startup::Event::Stdout(&startup_stdout[previous..]))?;
                }
                _ => {
                    if !expressions.is_empty() {
                        return Err(StartupError::Unsupported(
                            "use --args before expression arguments".into(),
                        ));
                    }
                    if matches!(mode, CliMode::File(_) | CliMode::Stdin) {
                        return Err(StartupError::Unsupported(
                            "use --args before file or stdin arguments".into(),
                        ));
                    }
                    let path = PathBuf::from(arg);
                    on_startup(startup::Event::File(&path))?;
                    mode = CliMode::File(path);
                    if args.peek().is_some_and(|arg| arg != "--args") {
                        command_args.insert(command_args.len() - args.len(), "--args".to_string());
                    }
                    break;
                }
            }
        }

        if matches!(mode, CliMode::Help | CliMode::Version) {
            startup_stdout.clear();
        } else if !expressions.is_empty() {
            if matches!(mode, CliMode::File(_)) {
                return Err(StartupError::Fatal(
                    "cannot use -e with -f or --file".into(),
                ));
            }
            expressions.pop();
            mode = CliMode::Expression(expressions);
        }
        Ok(Self {
            mode,
            quiet,
            no_echo,
            vanilla,
            no_save,
            command_args,
            startup_stdout,
        })
    }
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            mode: CliMode::Interactive,
            quiet: false,
            no_echo: false,
            vanilla: false,
            no_save: false,
            command_args: vec!["r_ust_cli".to_string()],
            startup_stdout: Vec::new(),
        }
    }
}
