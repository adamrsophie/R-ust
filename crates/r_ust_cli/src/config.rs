use std::path::PathBuf;

use super::startup;
use super::startup::StartupError;

mod backend;
mod shell;

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

    pub(super) fn parse_args<I, S, F>(args: I, on_startup: F) -> Result<Self, StartupError>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
        F: FnMut(startup::Event<'_>) -> Result<(), StartupError>,
    {
        let mut original = vec!["r_ust_cli".to_string()];
        original.extend(args.into_iter().map(Into::into));
        match shell::prepare(&original[1..])? {
            shell::Prepared::Help => Ok(Self {
                mode: CliMode::Help,
                command_args: original,
                ..Self::default()
            }),
            shell::Prepared::Arguments(args) => backend::parse(args, on_startup),
        }
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
