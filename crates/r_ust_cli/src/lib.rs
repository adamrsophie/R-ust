use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use r_ust_runtime::{Frontend, OutputEvent, PORT_TARGET};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CliMode {
    Interactive,
    File(PathBuf),
    Expression(String),
    Version,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CliConfig {
    pub mode: CliMode,
    pub quiet: bool,
    pub vanilla: bool,
}

impl CliConfig {
    pub fn from_args<I, S>(args: I) -> Result<Self, String>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut quiet = false;
        let mut vanilla = false;
        let mut mode = CliMode::Interactive;
        let mut args = args.into_iter().map(Into::into).peekable();

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--quiet" | "-q" => quiet = true,
                "--vanilla" => vanilla = true,
                "--version" | "version" => mode = CliMode::Version,
                "-e" => {
                    let expression = args
                        .next()
                        .ok_or_else(|| "-e requires an expression".to_string())?;
                    mode = CliMode::Expression(expression);
                }
                "--file" | "-f" => {
                    let file = args
                        .next()
                        .ok_or_else(|| format!("{arg} requires a file path"))?;
                    mode = CliMode::File(PathBuf::from(file));
                }
                "--args" => break,
                _ if arg.starts_with('-') => {
                    return Err(format!("unsupported CLI option {arg}"));
                }
                _ => {
                    mode = CliMode::File(PathBuf::from(arg));
                }
            }
        }

        Ok(Self {
            mode,
            quiet,
            vanilla,
        })
    }
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            mode: CliMode::Interactive,
            quiet: false,
            vanilla: false,
        }
    }
}

pub fn run(config: CliConfig) -> Result<(), String> {
    execute(config, Some(write_console)).map(|_| ())
}

/// Captures stdout. Console messages on stderr are available through runtime output events.
pub fn run_to_string(config: CliConfig) -> Result<String, String> {
    let output = execute(config, None)?;
    Ok(output
        .into_iter()
        .filter_map(|event| match event {
            OutputEvent::Stdout(text) => Some(text),
            OutputEvent::Stderr(_) => None,
        })
        .collect())
}

fn execute(
    config: CliConfig,
    sink: Option<fn(OutputEvent) -> io::Result<()>>,
) -> Result<Vec<OutputEvent>, String> {
    let source = match config.mode {
        CliMode::Version => {
            let event = OutputEvent::Stdout(format!("R-ust 0.1.0\n{PORT_TARGET}\n"));
            if let Some(sink) = sink {
                sink(event).map_err(|error| error.to_string())?;
                return Ok(Vec::new());
            }
            return Ok(vec![event]);
        }
        CliMode::Expression(source) => source,
        CliMode::File(path) => fs::read_to_string(&path)
            .map_err(|error| format!("failed to read {}: {error}", path.display()))?,
        CliMode::Interactive => {
            return Err(
                "interactive console is not implemented yet; use -e or a script file".to_string(),
            )
        }
    };
    let mut frontend = Frontend::default();
    if let Some(sink) = sink {
        frontend.runtime_mut().set_output_sink(sink);
    }
    frontend
        .eval_source_printed(&source)
        .map_err(|error| error.to_string())?;
    Ok(frontend.runtime_mut().take_output())
}

fn write_console(event: OutputEvent) -> io::Result<()> {
    match event {
        OutputEvent::Stdout(text) => {
            let mut stream = io::stdout().lock();
            stream.write_all(text.as_bytes())?;
            stream.flush()
        }
        OutputEvent::Stderr(text) => {
            let mut stream = io::stderr().lock();
            stream.write_all(text.as_bytes())?;
            stream.flush()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{run_to_string, CliConfig, CliMode};

    #[test]
    fn parses_expression_mode() {
        let config = CliConfig::from_args(["--vanilla", "-q", "-e", "1 + 1"]).unwrap();
        assert_eq!(
            config,
            CliConfig {
                mode: CliMode::Expression("1 + 1".to_string()),
                quiet: true,
                vanilla: true,
            }
        );
    }

    #[test]
    fn evaluates_expression_mode() {
        let output = run_to_string(CliConfig {
            mode: CliMode::Expression("x <- 1 + 1; x".to_string()),
            quiet: true,
            vanilla: true,
        })
        .unwrap();
        assert_eq!(output, "[1] 2\n");
    }
}
