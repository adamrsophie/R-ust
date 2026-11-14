use super::batch_command;

#[test]
fn expression_mode_runs_user_functions_and_return() {
    let output = batch_command()
        .args([
            "--vanilla",
            "-q",
            "-e",
            "abs2 <- function(x) { if (x < 0) return(-x); x }; abs2; abs2(-3); abs2(4)",
        ])
        .output()
        .expect("run r_ust_cli");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty(), "{:?}", output.stderr);
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert_eq!(
        stdout,
        "function (x) \n{\n    if (x < 0) \n        return(-x)\n    x\n}\n[1] 3\n[1] 4\n"
    );
}
