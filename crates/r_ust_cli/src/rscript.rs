use super::launcher::{append_expression, decode_argument, encode_expression};
use super::{run, startup, write_console, CliConfig, CliMode, OutputEvent, PORT_TARGET};

const HELP: &str = "Usage: Rscript [options] file [args]
   or: Rscript [options] -e expr [-e expr2 ...] [args]

Run R code with the R-ust interpreter, without requiring R.
Use - as the file to read stdin. Script execution is non-interactive.
Arguments after the file or expressions are available through commandArgs(TRUE).

  --help          Print usage and exit
  --version       Print version and compatibility target
  --verbose       Print the normalized invocation on stderr
  --quiet         Suppress startup messages
  --no-save       Exit without saving the workspace
  --vanilla       Disable startup files and workspace restoration; implies --no-save
  --no-echo       Suppress prompts and input echo (the default)
  --no-restore    Disable workspace restoration (the default)
  --no-environ    Disable environment files
  --no-site-file  Disable site startup files
  --no-init-file  Disable user startup files

R-ust is an early implementation. Package selection, startup file loading,
workspace saving/restoration, and full R option compatibility remain unfinished.
";

/// Runs the Rscript frontend with a complete invocation, including the program name.
pub fn run_rscript<I, S>(args: I) -> Result<i32, String>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut args = args.into_iter().map(Into::into);
    let program = args.next().unwrap_or_else(|| "Rscript".to_string());
    let mut args = args.collect::<Vec<_>>();
    // Rscript splits joined shebang options only when the first argument starts with --.
    if let Some(first) = args.first().filter(|arg| arg.starts_with("--")) {
        let split = first
            .split([' ', '\t'])
            .filter(|part| !part.is_empty())
            .map(str::to_string)
            .collect::<Vec<_>>();
        if split.len() > 1 {
            args.splice(..1, split);
        }
    }
    if args.is_empty() || args == ["--help"] {
        write_console(OutputEvent::Stdout(HELP.to_string())).map_err(|error| error.to_string())?;
        return Ok(i32::from(args.is_empty()));
    }
    if args == ["--version"] {
        write_console(OutputEvent::Stdout(format!(
            "Rscript (R-ust) {}\n{PORT_TARGET}\n",
            env!("CARGO_PKG_VERSION")
        )))
        .map_err(|error| error.to_string())?;
        return Ok(0);
    }

    let mut invocation = vec![program, "--no-echo".to_string(), "--no-restore".to_string()];
    let mut expressions = String::new();
    let mut expression_mode = false;
    let mut startup_stdout = Vec::new();
    let mut verbose = false;
    let mut vanilla = false;
    let mut file = None;
    let mut args = args.into_iter().peekable();
    while let Some(arg) = args.peek() {
        if arg == "-e" {
            let option = args.next().unwrap();
            let expression = args.next().ok_or("-e requires an expression")?;
            let encoded = encode_expression(&expression)?;
            expression_mode = true;
            append_expression(&mut expressions, &mut startup_stdout, &encoded);
            invocation.extend([option, encoded]);
        } else if expression_mode {
            break;
        } else if arg.starts_with("--") {
            let option = args.next().unwrap();
            match option.as_str() {
                "--verbose" => {
                    verbose = true;
                    continue;
                }
                "--vanilla" => vanilla = true,
                "--quiet" | "--silent" | "--no-save" => {}
                "--no-echo"
                | "--slave"
                | "--no-restore"
                | "--no-restore-data"
                | "--no-restore-history"
                | "--no-environ"
                | "--no-site-file"
                | "--no-init-file" => {}
                "--save" => return Err("workspace saving is not implemented yet".to_string()),
                "--restore" | "--restore-data" | "--restore-history" => {
                    return Err("workspace restoration is not implemented yet".to_string());
                }
                _ if option.starts_with("--default-packages=") => {
                    return Err("package selection is not implemented yet".to_string());
                }
                _ if startup::is_backend_option(&option) => {
                    return Err(format!("unsupported Rscript option {option}"));
                }
                _ => startup::append_unknown_option(&mut startup_stdout, &option),
            }
            invocation.push(option);
        } else {
            let path = args.next().unwrap();
            let encoded = path.replace(' ', "~+~");
            invocation.push(format!("--file={encoded}"));
            file = Some(decode_argument(&encoded));
            break;
        }
    }
    let mode = if expression_mode {
        if expressions.is_empty() {
            CliMode::Stdin
        } else {
            expressions.pop();
            CliMode::Expression(expressions)
        }
    } else {
        match file.ok_or("file name is missing")?.as_str() {
            "-" => CliMode::Stdin,
            path => CliMode::File(path.into()),
        }
    };
    let trailing = args.collect::<Vec<_>>();
    if !trailing.is_empty() {
        invocation.push("--args".to_string());
        invocation.extend(trailing.iter().cloned());
    }
    let config = CliConfig {
        mode,
        quiet: true,
        no_echo: true,
        no_save: true,
        vanilla,
        command_args: invocation,
        startup_stdout,
    };
    if verbose {
        write_console(OutputEvent::Stderr(format!(
            "R-ust Rscript arguments: {:?}\n",
            config.command_args
        )))
        .map_err(|error| error.to_string())?;
    }
    run(config)
}
