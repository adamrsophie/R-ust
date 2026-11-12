use std::fs;
use std::io::{self, IsTerminal, Read, Write};
use std::path::{Path, PathBuf};

mod launcher;
mod rscript;
mod startup;
pub use rscript::run_rscript;
pub use startup::run_main;
use startup::StartupError;

use r_ust_runtime::{
    EvalError, Frontend, OutputEvent, ParseStatus, Parser, QuitRequest, SaveAction, Sexp,
    PORT_TARGET,
};

const HELP: &str = "Usage: r_ust_cli [options] [script.R]

With no script or expression, start a console on a terminal or read piped stdin.
Console input supports multiple lines. Use q() or Ctrl-D to exit.
Workspace saving is not implemented; choose n or c at the save prompt.

  -e EXPR          Evaluate an expression; may be repeated
  -f, --file FILE  Run a script (also --file=FILE; --file=- reads stdin)
  -                Read stdin; use --no-echo to suppress input and prompts
  -q, --quiet      Suppress the startup banner (also --silent)
  -s, --no-echo    Suppress input echo, banner, and default save prompts (also --slave)
  --no-save        Exit without prompting to save the workspace
  --vanilla        Implies --no-save; startup files are not loaded
  --args ...       Pass remaining arguments to commandArgs(trailingOnly = TRUE)
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

    fn parse_args<I, S, F>(args: I, mut on_startup: F) -> Result<Self, StartupError>
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
                _ if arg.starts_with('-') => {
                    return Err(StartupError::Unsupported(format!(
                        "unsupported CLI option {arg}"
                    )));
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
        let input = opened.map(Ok).unwrap_or_else(|| fs::File::open(path));
        let source = match input {
            Ok(ref file) => read_open_script(file),
            Err(error) => Err(error),
        };
        let source = match source {
            Ok(source) => source,
            Err(error) if error.kind() == io::ErrorKind::InvalidData => {
                return Err(format!("failed to read {}: {error}", path.display()));
            }
            Err(error) => {
                write_console(OutputEvent::Stdout(format!(
                    "Fatal error: {}\n",
                    startup::file_error(path, &error)
                )))
                .map_err(|error| error.to_string())?;
                return Ok(2);
            }
        };
        return execute_source(&config, &source, Some(write_console)).map(|(_, status)| status);
    }
    execute(config, Some(write_console)).map(|(_, status)| status)
}

fn read_script(path: &Path) -> io::Result<String> {
    read_open_script(&fs::File::open(path)?)
}

fn read_open_script(mut file: &fs::File) -> io::Result<String> {
    let mut source = String::new();
    match file.read_to_string(&mut source) {
        Ok(_) => Ok(source),
        Err(error) if error.kind() == io::ErrorKind::IsADirectory => Ok(String::new()),
        Err(error) => Err(error),
    }
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
        read_console_line,
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
    mut read_line: impl FnMut() -> io::Result<String>,
) -> Result<i32, EvalError> {
    let mut source = String::new();
    'session: loop {
        let echo = echo_enabled(frontend);
        if echo {
            let key = if source.is_empty() {
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
        let mut line = read_line().map_err(|error| EvalError::Input(error.to_string()))?;
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
            let unterminated_cr = line.ends_with('\r');
            if line.ends_with("\r\n") {
                line.remove(line.len() - 2);
            }
            if !line.ends_with('\n') {
                line.push('\n');
            }
            if echo && !interactive {
                frontend
                    .runtime_mut()
                    .emit(OutputEvent::Stdout(line.clone()))?;
            }
            // Keep a bare CR at EOF distinct from an actual CRLF input pair.
            if unterminated_cr {
                line.pop();
            }
            source.push_str(&line);
            let mut statements = Parser.parse_iter(&source);
            let mut result = Ok(());
            while let Some(parsed) = statements.next() {
                match parsed {
                    Ok(parsed) => {
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
    Ok(startup
        + &output
            .into_iter()
            .filter_map(|event| match event {
                OutputEvent::Stdout(text) => Some(text),
                OutputEvent::Stderr(_) => None,
            })
            .collect::<String>())
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
        CliMode::File(path) => read_script(path)
            .map_err(|error| format!("failed to read {}: {error}", path.display()))?,
        CliMode::Interactive | CliMode::Stdin => {
            return Err(
                "run_to_string requires an expression, script file, help, or version; use run for stdin".into(),
            )
        }
    };
    execute_source(&config, &source, sink)
}

fn execute_source(
    config: &CliConfig,
    source: &str,
    sink: Option<fn(OutputEvent) -> io::Result<()>>,
) -> Result<(Vec<OutputEvent>, i32), String> {
    let mut frontend =
        initialize_frontend(config, false, sink).map_err(|error| error.to_string())?;
    let mut lines = source.split_inclusive('\n');
    let status = run_session(
        &mut frontend,
        false,
        config.no_save || config.vanilla,
        || Ok(lines.next().unwrap_or_default().to_string()),
    )
    .map_err(|error| error.to_string())?;
    Ok((frontend.runtime_mut().take_output(), status))
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
            Ok(String::new())
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
