#[path = "cli/character_printing.rs"]
mod character_printing;
use std::process::Command;

#[path = "cli/function_source.rs"]
mod function_source;

fn batch_command() -> Command {
    let binary = env!("CARGO_BIN_EXE_r_ust_cli");
    let mut command = Command::new(binary);
    command.arg("--no-echo");
    command
}

fn run_stdin(source: &str, args: &[&str]) -> std::process::Output {
    use std::io::Write;
    use std::process::Stdio;
    let mut child = batch_command()
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(source.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

#[path = "cli/data.rs"]
mod data;
#[path = "cli/distributions.rs"]
mod distributions;
#[path = "cli/effects.rs"]
mod effects;
#[path = "cli/language.rs"]
mod language;
#[path = "cli/matrices.rs"]
mod matrices;
#[path = "cli/numeric.rs"]
mod numeric;
#[path = "cli/session.rs"]
mod session;
#[path = "cli/vectors.rs"]
mod vectors;
