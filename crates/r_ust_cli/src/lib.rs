use std::fs;
use std::io::{self, BufRead, IsTerminal, Write};
use std::path::PathBuf;

use r_ust_runtime::{
    EvalError, Frontend, OutputEvent, ParseStatus, Parser, QuitRequest, SaveAction, PORT_TARGET,
};

const HELP: &str = "Usage: r_ust_cli [options] [script.R]

With no script or expression, start a console on a terminal or read piped stdin.
Console input supports multiple lines. Use q() or Ctrl-D to exit.
Workspace saving is not implemented; choose n or c at the save prompt.

  -e EXPR          Evaluate an expression
  -f, --file FILE  Run a script; use - for stdin
  -                Read stdin without prompts
  -q, --quiet      Suppress the console startup banner
  --no-save        Exit without prompting to save the workspace
  --vanilla        Implies --no-save; startup files are not loaded
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
    pub no_save: bool,
}

impl CliConfig {
    pub fn from_args<I, S>(args: I) -> Result<Self, String>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut quiet = false;
        let mut vanilla = false;
        let mut no_save = false;
        let mut mode = CliMode::Interactive;
        let mut args = args.into_iter().map(Into::into).peekable();

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--quiet" | "-q" => quiet = true,
                "--vanilla" => {
                    vanilla = true;
                    no_save = true;
                }
                "--no-save" => no_save = true,
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
            no_save,
        })
    }
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            mode: CliMode::Interactive,
            quiet: false,
            vanilla: false,
            no_save: false,
        }
    }
}

pub fn run(config: CliConfig) -> Result<i32, String> {
    if matches!(config.mode, CliMode::Interactive | CliMode::Stdin) {
        return run_input(config);
    }
    execute(config, Some(write_console)).map(|(_, status)| status)
}

fn run_input(config: CliConfig) -> Result<i32, String> {
    // shortcut: canonical terminal input; add a line editor for history and interrupt recovery.
    let input = io::stdin();
    let interactive = input.is_terminal() && config.mode == CliMode::Interactive;
    let mut input = input.lock();
    let mut frontend = Frontend::default();
    frontend.runtime_mut().set_output_sink(write_console);
    if interactive && !config.quiet {
        write_console(OutputEvent::Stdout(format!(
            "R-ust 0.1.0\n{PORT_TARGET}\nUse q() or Ctrl-D to exit.\n"
        )))
        .map_err(|error| error.to_string())?;
    }
    let mut source = String::new();
    'session: loop {
        if interactive {
            write_console(OutputEvent::Stdout(
                if source.is_empty() { "> " } else { "+ " }.to_string(),
            ))
            .map_err(|error| error.to_string())?;
        }
        let result = if input
            .read_line(&mut source)
            .map_err(|error| error.to_string())?
            == 0
        {
            if !source.is_empty() {
                return Err("parse error: unexpected end of input".to_string());
            }
            Err(EvalError::Quit(QuitRequest {
                save: SaveAction::Default,
                status: 0,
                run_last: true,
            }))
        } else {
            let mut statements = Parser.parse_iter(&source);
            let mut result = Ok(());
            while let Some(parsed) = statements.next() {
                match parsed {
                    Ok(parsed) => {
                        if let Err(error) = parsed
                            .expressions
                            .iter()
                            .try_for_each(|expr| frontend.eval_printed(expr))
                        {
                            result = Err(error);
                            break;
                        }
                    }
                    Err(error) if error.status == ParseStatus::Incomplete => {
                        let consumed = statements.consumed_bytes();
                        drop(statements);
                        source.drain(..consumed);
                        continue 'session;
                    }
                    Err(error) => {
                        result = Err(EvalError::Parse(error.message));
                        break;
                    }
                }
            }
            result
        };
        let result = match result {
            Err(EvalError::Quit(request)) => match complete_quit(
                &mut frontend,
                request,
                interactive,
                config.no_save || config.vanilla,
                &mut input,
            ) {
                Ok(Some(status)) => return Ok(status),
                Ok(None) => Ok(()),
                Err(error) => Err(error),
            },
            other => other,
        };
        if let Err(error) = result {
            if !interactive || matches!(error, EvalError::Output(_)) {
                return Err(error.to_string());
            }
            write_console(OutputEvent::Stderr(format!("Error: {error}\n")))
                .map_err(|error| error.to_string())?;
        }
        source.clear();
    }
}

fn complete_quit(
    frontend: &mut Frontend,
    mut request: QuitRequest,
    interactive: bool,
    no_save: bool,
    input: &mut dyn BufRead,
) -> Result<Option<i32>, EvalError> {
    let output_error = |error: io::Error| EvalError::Output(error.to_string());
    loop {
        let mut save = match request.save {
            SaveAction::Default if interactive && !no_save => SaveAction::Ask,
            SaveAction::Default => SaveAction::No,
            action => action,
        };
        if save == SaveAction::Ask {
            if interactive {
                loop {
                    write_console(OutputEvent::Stdout(
                        "Save workspace image? [y/n/c]: ".to_string(),
                    ))
                    .map_err(output_error)?;
                    let mut answer = String::new();
                    let read = input.read_line(&mut answer).map_err(output_error)?;
                    match answer.chars().next() {
                        Some('y' | 'Y') => {
                            save = SaveAction::Yes;
                            break;
                        }
                        Some('n' | 'N') => {
                            save = SaveAction::No;
                            break;
                        }
                        Some('c' | 'C') => return Ok(None),
                        _ if read == 0 => {
                            save = SaveAction::No;
                            break;
                        }
                        _ => {}
                    }
                }
            } else {
                write_console(OutputEvent::Stderr("Warning: save=\"ask\" in non-interactive use: command-line default will be used\n".to_string())).map_err(output_error)?;
                save = SaveAction::No;
            }
        }
        if save == SaveAction::Yes {
            return Err(EvalError::NotImplemented("workspace saving"));
        }
        if request.run_last {
            match frontend.runtime_mut().run_last() {
                Err(EvalError::Quit(next)) => {
                    request = next;
                    continue;
                }
                result => result?,
            }
        }
        return Ok(Some(request.status));
    }
}

/// Captures stdout, including explicit quit requests. Use `run` for process exit status.
pub fn run_to_string(config: CliConfig) -> Result<String, String> {
    let (output, _) = execute(config, None)?;
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
) -> Result<(Vec<OutputEvent>, i32), String> {
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
                return Ok((Vec::new(), 0));
            }
            return Ok((vec![event], 0));
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
    let status = match frontend.eval_source_printed(&source) {
        Ok(()) => complete_quit(
            &mut frontend,
            QuitRequest {
                save: SaveAction::No,
                status: 0,
                run_last: true,
            },
            false,
            true,
            &mut io::empty(),
        ),
        Err(EvalError::Quit(request)) => {
            complete_quit(&mut frontend, request, false, true, &mut io::empty())
        }
        Err(error) => return Err(error.to_string()),
    }
    .map_err(|error| error.to_string())?
    .unwrap_or(0);
    Ok((frontend.runtime_mut().take_output(), status))
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
                no_save: true,
            }
        );
    }

    #[test]
    fn evaluates_expression_mode() {
        let output = run_to_string(CliConfig {
            mode: CliMode::Expression("x <- 1 + 1; x".to_string()),
            quiet: true,
            vanilla: true,
            no_save: true,
        })
        .unwrap();
        assert_eq!(output, "[1] 2\n");
    }
}
