use std::fs;
use std::io::{self, BufRead, IsTerminal, Read, Write};

mod config;
mod launcher;
mod rscript;
mod startup;
pub use config::{CliConfig, CliMode};
pub use rscript::run_rscript;
pub use startup::run_main;

use r_ust_runtime::{
    EvalError, Frontend, OutputEvent, ParseStatus, Parser, QuitRequest, SaveAction, Sexp,
    PORT_TARGET,
};

const HELP: &str = "Usage: r_ust_cli [options] [< infile]

With no -f or -e input, start a console on a terminal or read piped stdin.
Console input supports multiple lines. Use q() or Ctrl-D to exit.
Workspace saving is not implemented; choose n or c at the save prompt.

  -e EXPR          Evaluate an expression; may be repeated
  -f FILE          Run a script (also --file=FILE)
  --file=-         Read stdin; use --no-echo to suppress input and prompts
  -q, --quiet      Suppress the startup banner (also --silent)
  -s, --no-echo    Suppress input echo, banner, and default save prompts (also --slave)
  --no-save        Exit without prompting to save the workspace
  --vanilla        Implies --no-save; startup files are not loaded
  --args ...       Pass remaining arguments to commandArgs(trailingOnly = TRUE)
  --version        Show version and R compatibility target
  -h, --help       Show this help
";

const CONSOLE_CHUNK_BYTES: usize = 4095;

pub fn run(config: CliConfig) -> Result<i32, String> {
    run_with_file(config, None)
}

fn run_with_file(config: CliConfig, opened: Option<fs::File>) -> Result<i32, String> {
    if !config.startup_stdout.is_empty() {
        let mut stdout = io::stdout().lock();
        stdout
            .write_all(&config.startup_stdout)
            .and_then(|_| stdout.flush())
            .map_err(|error| error.to_string())?;
    }
    if matches!(config.mode, CliMode::Interactive | CliMode::Stdin) {
        return run_input(config);
    }
    if let CliMode::File(path) = &config.mode {
        let input = match opened.map(Ok).unwrap_or_else(|| fs::File::open(path)) {
            Ok(file) => file,
            Err(error) => {
                write_console(OutputEvent::Stdout(format!(
                    "Fatal error: {}\n",
                    startup::file_error(path, &error)
                )))
                .map_err(|error| error.to_string())?;
                return Ok(2);
            }
        };
        return execute_reader(&config, io::BufReader::new(&input), Some(write_console))
            .map(|(_, status)| status);
    }
    execute(config, Some(write_console)).map(|(_, status)| status)
}

fn initialize_frontend(
    config: &CliConfig,
    interactive: bool,
    sink: Option<fn(OutputEvent) -> io::Result<()>>,
) -> Result<Frontend, EvalError> {
    let mut frontend = Frontend::default();
    frontend
        .runtime_mut()
        .set_session_context(config.command_args.clone(), interactive);
    if let Some(sink) = sink {
        frontend.runtime_mut().set_output_sink(sink);
    }
    frontend.runtime_mut().set_input_reader(read_console_line);
    frontend
        .runtime_mut()
        .set_option("quiet", Sexp::Logical(vec![Some(config.quiet)]))?;
    frontend
        .runtime_mut()
        .set_option("echo", Sexp::Logical(vec![Some(!config.no_echo)]))?;
    if !config.quiet {
        frontend.runtime_mut().emit(OutputEvent::Stdout(format!(
            "R-ust 0.1.0\n{PORT_TARGET}\nUse q() or Ctrl-D to exit.\n"
        )))?;
    }
    Ok(frontend)
}

fn run_input(config: CliConfig) -> Result<i32, String> {
    let interactive = io::stdin().is_terminal() && config.mode == CliMode::Interactive;
    let mut frontend = initialize_frontend(&config, interactive, Some(write_console))
        .map_err(|error| error.to_string())?;
    run_session(
        &mut frontend,
        interactive,
        config.no_save || config.vanilla,
        || read_source_chunk(&mut io::stdin().lock()),
    )
    .map_err(|error| error.to_string())
}

fn echo_enabled(frontend: &Frontend) -> bool {
    !matches!(frontend.runtime().get_option("echo"), Some(Sexp::Logical(values)) if values.first() == Some(&Some(false)))
}

fn run_session(
    frontend: &mut Frontend,
    interactive: bool,
    no_save: bool,
    mut read_chunk: impl FnMut() -> io::Result<Vec<u8>>,
) -> Result<i32, EvalError> {
    let mut source = Vec::new();
    let mut blank_continuation = false;
    'session: loop {
        let echo = echo_enabled(frontend);
        if echo {
            let key = if source.is_empty() && !blank_continuation {
                "prompt"
            } else {
                "continue"
            };
            let prompt = match frontend.runtime().get_option(key) {
                Some(Sexp::String(values)) => values.first().and_then(Option::as_ref).cloned(),
                _ => None,
            }
            .ok_or_else(|| EvalError::Type(format!("invalid '{key}' option")))?;
            frontend.runtime_mut().emit(OutputEvent::Stdout(prompt))?;
        }
        let mut line = read_chunk().map_err(|error| EvalError::Input(error.to_string()))?;
        blank_continuation = false;
        let result = if line.is_empty() {
            if !source.is_empty() {
                return Err(EvalError::Parse("unexpected end of input".into()));
            }
            if echo {
                frontend
                    .runtime_mut()
                    .emit(OutputEvent::Stdout("\n".into()))?;
            }
            Err(EvalError::Quit(QuitRequest {
                save: SaveAction::Default,
                status: 0,
                run_last: true,
            }))
        } else {
            let full = line.len() == CONSOLE_CHUNK_BYTES && !line.ends_with(b"\n");
            let unterminated_cr = !full && line.ends_with(b"\r");
            if line.ends_with(b"\r\n") {
                line.remove(line.len() - 2);
            }
            if !full && !line.ends_with(b"\n") {
                line.push(b'\n');
            }
            if echo && !interactive {
                let event = match String::from_utf8(line.clone()) {
                    Ok(text) => OutputEvent::Stdout(text),
                    Err(error) => OutputEvent::StdoutBytes(error.into_bytes()),
                };
                frontend.runtime_mut().emit(event)?;
            }
            // Keep a bare CR at EOF distinct from an actual CRLF input pair.
            if unterminated_cr {
                line.pop();
            }
            source.extend_from_slice(&line);
            let (text, invalid) = match std::str::from_utf8(&source) {
                Ok(text) => (text, None),
                Err(error) => (
                    std::str::from_utf8(&source[..error.valid_up_to()])
                        .expect("valid UTF-8 prefix"),
                    Some(error),
                ),
            };
            let mut statements = if unterminated_cr {
                Parser.parse_iter(text)
            } else {
                Parser.parse_iter_open(text)
            };
            let mut result = Ok(());
            while let Some(parsed) = statements.next() {
                match parsed {
                    Ok(parsed) => {
                        blank_continuation = full && parsed.status == ParseStatus::Null;
                        let warnings = frontend
                            .runtime_mut()
                            .record_parse_warnings(&parsed.warnings);
                        if let Err(error) = warnings.and_then(|_| {
                            parsed
                                .expressions
                                .iter()
                                .try_for_each(|expr| frontend.eval_printed(expr))
                        }) {
                            result = Err(error);
                            break;
                        }
                    }
                    Err(error) => {
                        if error.status == ParseStatus::Incomplete {
                            if invalid.is_some_and(|error| error.error_len().is_some()) {
                                result = Err(EvalError::Input(
                                    "stream did not contain valid UTF-8".into(),
                                ));
                                break;
                            }
                            let consumed = statements.consumed_bytes();
                            drop(statements);
                            source.drain(..consumed);
                            continue 'session;
                        }
                        result = Err(EvalError::Parse(error.message));
                        break;
                    }
                }
            }
            if result.is_ok() {
                if let Some(error) = invalid {
                    if error.error_len().is_some() {
                        result = Err(EvalError::Input(
                            "stream did not contain valid UTF-8".into(),
                        ));
                    } else {
                        let consumed = statements.consumed_bytes();
                        drop(statements);
                        source.drain(..consumed);
                        continue 'session;
                    }
                }
            }
            result
        };
        let result = match result {
            Err(EvalError::Quit(request)) => {
                let cleanup =
                    complete_quit(frontend, request, interactive, no_save, read_console_line);
                match frontend.runtime_mut().finish_parse_warnings(cleanup) {
                    Ok(Some(status)) => return Ok(status),
                    Ok(None) => Ok(()),
                    Err(error) => Err(error),
                }
            }
            other => other,
        };
        if let Err(error) = result {
            if !interactive || matches!(error, EvalError::Output(_) | EvalError::Input(_)) {
                return Err(error);
            }
            frontend
                .runtime_mut()
                .emit(OutputEvent::Stderr(format!("Error: {error}\n")))?;
        }
        source.clear();
    }
}

fn complete_quit(
    frontend: &mut Frontend,
    mut request: QuitRequest,
    interactive: bool,
    no_save: bool,
    read_line: fn() -> io::Result<String>,
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
                    let answer =
                        read_line().map_err(|error| EvalError::Input(error.to_string()))?;
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
                        _ if answer.is_empty() => {
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
    let startup = String::from_utf8(config.startup_stdout.clone())
        .map_err(|_| "startup output is not UTF-8; use run to stream raw bytes".to_string())?;
    let (output, _) = execute(config, None)?;
    let mut bytes = startup.into_bytes();
    for event in output {
        match event {
            OutputEvent::Stdout(text) => bytes.extend_from_slice(text.as_bytes()),
            OutputEvent::StdoutBytes(raw) => bytes.extend(raw),
            OutputEvent::Stderr(_) => {}
        }
    }
    String::from_utf8(bytes)
        .map_err(|_| "console output is not UTF-8; use run to stream raw bytes".into())
}

fn execute(
    config: CliConfig,
    sink: Option<fn(OutputEvent) -> io::Result<()>>,
) -> Result<(Vec<OutputEvent>, i32), String> {
    let source = match &config.mode {
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
        CliMode::Expression(source) => format!("{source}\n"),
        CliMode::File(path) => {
            let file = fs::File::open(path)
                .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
            return execute_reader(&config, io::BufReader::new(file), sink);
        }
        CliMode::Interactive | CliMode::Stdin => {
            return Err(
                "run_to_string requires an expression, script file, help, or version; use run for stdin".into(),
            )
        }
    };
    execute_reader(&config, io::Cursor::new(source), sink)
}

fn execute_reader(
    config: &CliConfig,
    mut reader: impl BufRead,
    sink: Option<fn(OutputEvent) -> io::Result<()>>,
) -> Result<(Vec<OutputEvent>, i32), String> {
    let mut frontend =
        initialize_frontend(config, false, sink).map_err(|error| error.to_string())?;
    let status = run_session(
        &mut frontend,
        false,
        config.no_save || config.vanilla,
        || read_source_chunk(&mut reader),
    )
    .map_err(|error| error.to_string())?;
    Ok((frontend.runtime_mut().take_output(), status))
}

fn read_source_chunk(reader: &mut impl BufRead) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    match reader
        .take(CONSOLE_CHUNK_BYTES as u64)
        .read_until(b'\n', &mut bytes)
    {
        Ok(_) => Ok(bytes),
        Err(error) if error.kind() == io::ErrorKind::IsADirectory => Ok(Vec::new()),
        Err(error) => Err(error),
    }
}

fn read_console_line() -> io::Result<String> {
    // shortcut: canonical terminal input; add a line editor for history and interrupt recovery.
    let mut line = String::new();
    io::stdin().read_line(&mut line)?;
    Ok(line)
}

fn write_console(event: OutputEvent) -> io::Result<()> {
    match event {
        OutputEvent::Stdout(text) => {
            let mut stream = io::stdout().lock();
            stream.write_all(text.as_bytes())?;
            stream.flush()
        }
        OutputEvent::StdoutBytes(bytes) => {
            let mut stream = io::stdout().lock();
            stream.write_all(&bytes)?;
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
                no_echo: false,
                vanilla: true,
                no_save: true,
                command_args: vec!["r_ust_cli", "--vanilla", "-q", "-e", "1~+~+~+~1"]
                    .into_iter()
                    .map(str::to_string)
                    .collect(),
                startup_stdout: Vec::new(),
            }
        );
    }

    #[test]
    fn evaluates_expression_mode() {
        let output = run_to_string(CliConfig {
            mode: CliMode::Expression("x <- 1 + 1; x".to_string()),
            quiet: true,
            no_echo: true,
            vanilla: true,
            no_save: true,
            ..CliConfig::default()
        })
        .unwrap();
        assert_eq!(output, "[1] 2\n");
    }

    #[test]
    fn prompt_output_failure_prevents_input_reads() {
        let mut frontend = r_ust_runtime::Frontend::default();
        frontend
            .runtime_mut()
            .set_option("echo", r_ust_runtime::Sexp::Logical(vec![Some(true)]))
            .unwrap();
        frontend
            .runtime_mut()
            .set_output_sink(|_| Err(std::io::Error::other("closed output")));
        let mut read = false;
        let result = super::run_session(&mut frontend, false, true, || {
            read = true;
            Ok(Vec::new())
        });
        assert!(matches!(result, Err(r_ust_runtime::EvalError::Output(_))));
        assert!(!read);
    }

    #[test]
    fn source_input_failure_preserves_error_kind() {
        let mut frontend = r_ust_runtime::Frontend::default();
        let result = super::run_session(&mut frontend, false, true, || {
            Err(std::io::Error::other("closed input"))
        });
        assert!(matches!(result, Err(r_ust_runtime::EvalError::Input(_))));
    }
}
