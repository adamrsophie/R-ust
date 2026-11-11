use std::fs;
use std::path::PathBuf;

use r_ust_runtime::{format_sexp, Frontend, Sexp, PORT_TARGET};

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
    let output = run_to_string(config)?;
    print!("{output}");
    Ok(())
}

pub fn run_to_string(config: CliConfig) -> Result<String, String> {
    match config.mode {
        CliMode::Version => Ok(format!("R-ust 0.1.0\n{PORT_TARGET}\n")),
        CliMode::Expression(source) => eval_to_string(&source),
        CliMode::File(path) => {
            let source = fs::read_to_string(&path)
                .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
            eval_to_string(&source)
        }
        CliMode::Interactive => {
            Err("interactive console is not implemented yet; use -e or a script file".to_string())
        }
    }
}

fn eval_to_string(source: &str) -> Result<String, String> {
    let mut frontend = Frontend::default();
    let values = frontend
        .eval_source_visible(source)
        .map_err(|error| error.to_string())?;
    Ok(format_values(&values))
}

fn format_values(values: &[Sexp]) -> String {
    values
        .iter()
        .filter(|value| !matches!(value, Sexp::Nil))
        .map(format_sexp)
        .map(|line| format!("{line}\n"))
        .collect()
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
