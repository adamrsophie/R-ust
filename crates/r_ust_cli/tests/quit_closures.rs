use std::process::Command;

type Result = std::result::Result<(), Box<dyn std::error::Error>>;

fn check(source: &str, status: i32, stdout: &str) -> Result {
    let output = Command::new(env!("CARGO_BIN_EXE_r_ust_cli"))
        .args(["--vanilla", "--slave", "-e", source])
        .output()?;
    assert_eq!(output.status.code(), Some(status), "{source}: {output:?}");
    assert_eq!(String::from_utf8(output.stdout)?, stdout, "{source}");
    assert!(output.stderr.is_empty(), "{source}: {:?}", output.stderr);
    Ok(())
}

#[test]
fn quit_closures_cli_aliases_and_higher_order_calls_exit() -> Result {
    for source in [
        "f<-quit;f('no',7,FALSE);99",
        "f<-q;f('no',7,FALSE);99",
        "lapply('no',quit,status=7,runLast=FALSE);99",
        "f<-function(...)q(...);f('no',st=7,run=FALSE);99",
    ] {
        check(source, 7, "")?;
    }
    Ok(())
}

#[test]
fn quit_closures_cli_forcing_order_and_handler_escape() -> Result {
    check(
        "q(runLast={cat('R|');FALSE},status={cat('T|');9},save={cat('S|');'no'})",
        9,
        "S|T|R|",
    )?;
    check(
        "tryCatch(q('no',4,FALSE),error=function(e)cat('BAD'));99",
        4,
        "",
    )
}

#[test]
fn quit_closures_cli_alias_respects_last_and_status_wrapping() -> Result {
    check(
        ".Last<-function()cat('LAST|');f<-quit;f('no',257)",
        1,
        "LAST|",
    )?;
    check(".Last<-function()cat('BAD');f<-q;f('no',-1,FALSE)", 255, "")
}
