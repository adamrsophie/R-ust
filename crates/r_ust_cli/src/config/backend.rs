use std::path::PathBuf;

use super::super::{
    launcher,
    startup::{self, StartupError},
};
use super::{CliConfig, CliMode};

pub(super) fn parse<F>(args: Vec<String>, mut on_startup: F) -> Result<CliConfig, StartupError>
where
    F: FnMut(startup::Event<'_>) -> Result<(), StartupError>,
{
    let mut config = CliConfig::default();
    config.command_args.extend(args);
    let mut native = config.command_args.clone();
    let mut argc = 1;
    let mut processing = true;
    for read in 1..native.len() {
        let arg = native[read].clone();
        if processing {
            match arg.as_str() {
                "--version" => {
                    config.mode = CliMode::Version;
                    return Ok(config);
                }
                "--quiet" | "--silent" | "-q" => {
                    config.quiet = true;
                    continue;
                }
                "--no-echo" | "--slave" | "-s" => {
                    config.quiet = true;
                    config.no_echo = true;
                    config.no_save = true;
                    continue;
                }
                "--vanilla" => {
                    config.vanilla = true;
                    config.no_save = true;
                    continue;
                }
                "--no-save" => {
                    config.no_save = true;
                    continue;
                }
                "--no-restore"
                | "--no-restore-data"
                | "--no-restore-history"
                | "--no-environ"
                | "--no-site-file"
                | "--no-init-file" => continue,
                "--args" => processing = false,
                "-save" | "-nosave" | "-restore" | "-norestore" | "-noreadline" | "-quiet"
                | "-nsize" | "-vsize" | "-V" | "-n" | "-v" => {
                    let start = config.startup_stdout.len();
                    startup::append_message(
                        &mut config.startup_stdout,
                        &format!("WARNING: option '{arg}' no longer supported"),
                        1023,
                    );
                    on_startup(startup::Event::Stdout(&config.startup_stdout[start..]))?;
                    continue;
                }
                _ if arg.starts_with("--max-nsize") || arg.starts_with("--max-vsize") => {
                    let start = config.startup_stdout.len();
                    startup::append_message(
                        &mut config.startup_stdout,
                        &format!("WARNING: option '{arg}' no longer supported"),
                        1023,
                    );
                    on_startup(startup::Event::Stdout(&config.startup_stdout[start..]))?;
                    continue;
                }
                _ => {}
            }
        }
        native[argc] = arg;
        argc += 1;
    }
    let mut source = String::new();
    let mut index = 1;
    while index < argc {
        let arg = &native[index];
        match arg.as_str() {
            "--args" => break,
            "--no-readline" => {}
            "-e" => {
                index += 1;
                let encoded = native[..argc].get(index).ok_or_else(|| {
                    StartupError::Fatal("missing expression after launcher expansion".into())
                })?;
                let start = config.startup_stdout.len();
                launcher::append_expression(&mut source, &mut config.startup_stdout, encoded);
                if config.startup_stdout.len() != start {
                    on_startup(startup::Event::Stdout(&config.startup_stdout[start..]))?;
                }
                if !matches!(config.mode, CliMode::File(_)) {
                    config.mode = CliMode::Stdin;
                }
            }
            _ if arg == "-f" || arg.starts_with("--file=") => {
                let encoded = if let Some(file) = arg.strip_prefix("--file=") {
                    file
                } else {
                    index += 1;
                    native.get(index).map(String::as_str).ok_or_else(|| {
                        StartupError::Fatal("missing filename after launcher expansion".into())
                    })?
                };
                if encoded.len() >= 1024 {
                    return Err(StartupError::Fatal(
                        "path given in -f/--file is too long".into(),
                    ));
                }
                let file = launcher::decode_argument(encoded);
                if file == "-" {
                    if !matches!(config.mode, CliMode::File(_)) {
                        config.mode = CliMode::Stdin;
                    }
                } else {
                    let path = PathBuf::from(file);
                    on_startup(startup::Event::File(&path))?;
                    config.mode = CliMode::File(path);
                }
                if index >= argc {
                    return Err(StartupError::Fatal(
                        "missing filename after launcher expansion".into(),
                    ));
                }
            }
            _ if arg.starts_with('-') && startup::is_backend_option(arg) => {
                return Err(StartupError::Unsupported(format!(
                    "unsupported CLI option {arg}"
                )));
            }
            _ if arg.starts_with('-') => {
                let start = config.startup_stdout.len();
                startup::append_unknown_option(&mut config.startup_stdout, arg);
                on_startup(startup::Event::Stdout(&config.startup_stdout[start..]))?;
            }
            _ => {
                let start = config.startup_stdout.len();
                startup::append_message(
                    &mut config.startup_stdout,
                    &format!("ARGUMENT '{arg}' __ignored__\n"),
                    1151,
                );
                on_startup(startup::Event::Stdout(&config.startup_stdout[start..]))?;
            }
        }
        index += 1;
    }
    if !source.is_empty() {
        if matches!(config.mode, CliMode::File(_)) {
            return Err(StartupError::Fatal(
                "cannot use -e with -f or --file".into(),
            ));
        }
        source.pop();
        config.mode = CliMode::Expression(source);
    }
    Ok(config)
}
