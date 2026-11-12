use std::fs;
use std::io::{self, BufRead, IsTerminal, Write};
use std::path::PathBuf;

use r_ust_runtime::{EvalError, Frontend, OutputEvent, ParseStatus, Parser, PORT_TARGET};

const HELP: &str = "Usage: r_ust_cli [options] [script.R]

With no script or expression, start a console on a terminal or read piped stdin.
Console input supports multiple lines. Press Ctrl-D to exit.

  -e EXPR          Evaluate an expression
  -f, --file FILE  Run a script; use - for stdin
  -                Read stdin without prompts
  -q, --quiet      Suppress the console startup banner
  --vanilla        Accepted for compatibility; startup files are not loaded
  --version        Show version and R compatibility target
  -h, --help       Show this help
";

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
                "--help" | "-h" => mode = CliMode::Help,
                "-" => mode = CliMode::Stdin,
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
                    mode = if file == "-" {
                        CliMode::Stdin
                    } else {
                        CliMode::File(PathBuf::from(file))
                    };
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
    if matches!(config.mode, CliMode::Interactive | CliMode::Stdin) {
        return run_input(config);
    }
    execute(config, Some(write_console)).map(|_| ())
}

fn run_input(config: CliConfig) -> Result<(), String> {
    // shortcut: canonical terminal input; add a line editor for history and interrupt recovery.
    let input = io::stdin();
    let interactive = input.is_terminal() && config.mode == CliMode::Interactive;
    let mut input = input.lock();
    let mut frontend = Frontend::default();
    frontend.runtime_mut().set_output_sink(write_console);
    if interactive && !config.quiet {
        write_console(OutputEvent::Stdout(format!(
            "R-ust 0.1.0\n{PORT_TARGET}\nPress Ctrl-D to exit.\n"
        )))
        .map_err(|error| error.to_string())?;
    }
    let mut source = String::new();
    loop {
        if interactive {
            write_console(OutputEvent::Stdout(
                if source.is_empty() { "> " } else { "+ " }.to_string(),
            ))
            .map_err(|error| error.to_string())?;
        }
        if input
            .read_line(&mut source)
            .map_err(|error| error.to_string())?
            == 0
        {
            return if source.is_empty() {
                Ok(())
            } else {
                Err("parse error: unexpected end of input".to_string())
            };
        }
        let result = match Parser.parse_all(&source) {
            Ok(_) => match frontend.eval_source_printed(&source) {
                Err(error @ EvalError::Output(_)) => return Err(error.to_string()),
                result => result.map_err(|error| error.to_string()),
            },
            Err(error) if error.status == ParseStatus::Incomplete => continue,
            Err(error) => Err(format!("parse error: {}", error.message)),
        };
        if let Err(error) = result {
            if !interactive {
                return Err(error);
            }
            write_console(OutputEvent::Stderr(format!("Error: {error}\n")))
                .map_err(|error| error.to_string())?;
        }
        source.clear();
    }
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
        CliMode::Version | CliMode::Help => {
            let text = if config.mode == CliMode::Help {
                HELP.to_string()
            } else {
                format!("R-ust 0.1.0\n{PORT_TARGET}\n")
            };
            let event = OutputEvent::Stdout(text);
            if let Some(sink) = sink {
                sink(event).map_err(|error| error.to_string())?;
                return Ok(Vec::new());
            }
            return Ok(vec![event]);
        }
        CliMode::Expression(source) => source,
        CliMode::File(path) => fs::read_to_string(&path)
            .map_err(|error| format!("failed to read {}: {error}", path.display()))?,
        CliMode::Interactive | CliMode::Stdin => {
            return Err(
                "run_to_string requires an expression, script file, help, or version; use run for stdin".to_string(),
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
