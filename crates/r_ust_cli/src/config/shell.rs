use super::super::{launcher, startup::StartupError};

pub(super) enum Prepared {
    Help,
    Arguments(Vec<String>),
}

pub(super) fn prepare(args: &[String]) -> Result<Prepared, StartupError> {
    let mut output = Vec::new();
    let mut index = 0;
    while let Some(arg) = args.get(index) {
        let encoded = match arg.as_str() {
            "--help" | "-h" => return Ok(Prepared::Help),
            "--args" | "" => {
                output = launcher::expand_arguments(output.iter().map(String::as_str))
                    .map_err(StartupError::Expression)?;
                output.extend_from_slice(&args[index..]);
                return Ok(Prepared::Arguments(output));
            }
            "-e" => {
                index += 1;
                let value = args.get(index).ok_or(StartupError::Argument("-e"))?;
                output.push("-e".into());
                launcher::encode_expression(value).map_err(StartupError::Expression)?
            }
            "-f" => {
                index += 1;
                let value = args.get(index).ok_or(StartupError::Argument("-f"))?;
                if !launcher::valid_argument(value).map_err(StartupError::Expression)? {
                    return Err(StartupError::Argument("-f"));
                }
                output.push("-f".into());
                launcher::encode_filename(value, false).map_err(|error| {
                    StartupError::Unsupported(format!("filename launcher conversion: {error}"))
                })?
            }
            "CMD" | "RHOME" | "--print-home" => {
                return Err(StartupError::Unsupported(format!(
                    "{arg} is not implemented yet"
                )));
            }
            _ if arg.starts_with("--file=") => {
                let file = launcher::encode_filename(arg, true).map_err(|error| {
                    StartupError::Unsupported(format!("filename launcher conversion: {error}"))
                })?;
                format!("--file={file}")
            }
            _ => arg.clone(),
        };
        output.extend(launcher::fields(&encoded).map(str::to_string));
        index += 1;
    }
    let output = launcher::expand_arguments(output.iter().map(String::as_str))
        .map_err(StartupError::Expression)?;
    Ok(Prepared::Arguments(output))
}
