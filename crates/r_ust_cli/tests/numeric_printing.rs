use std::process::Command;

#[test]
fn expression_mode_runs_coercion_and_type_predicates() {
    let output = Command::new(env!("CARGO_BIN_EXE_r_ust_cli"))
        .arg("--no-echo")
        .args([
            "--vanilla",
            "-q",
            "-e",
            "as.integer(c(1.9, -1.9, NA, Inf)); as.double(as.integer(c(1, NA))); as.real(c(TRUE, FALSE, NA)); is.integer(as.integer(1)); is.double(c(1, 2)); is.real(c(1, 2)); is.atomic(c(1, 2)); is.atomic(list(1)); is.vector(c(1, 2)); is.vector(matrix(1:4, nrow = 2))",
        ])
        .output()
        .expect("run r_ust_cli");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains("[1]  1 -1 NA NA\n"));
    assert!(stdout.contains("[1]  1 NA\n"));
    assert!(stdout.contains("[1]  1  0 NA\n"));
    assert!(stdout.contains("[1] TRUE\n"));
    assert!(stdout.contains("[1] FALSE\n"));
}

#[test]
fn numeric_vectors_and_typed_empty_values_print_like_r() {
    for binary in [
        env!("CARGO_BIN_EXE_r_ust_cli"),
        env!("CARGO_BIN_EXE_Rscript"),
    ] {
        let output = Command::new(binary)
            .args(["--vanilla", "--no-echo", "-e", "as.numeric();as.double();as.integer();as.logical(NULL);as.integer(c(1,-10,NA));as.double(c(1,-10,NA));c(TRUE,FALSE,NA)"])
            .output().unwrap();
        assert!(output.status.success(), "{binary}: {output:?}");
        assert_eq!(String::from_utf8(output.stdout).unwrap(), "numeric(0)\nnumeric(0)\ninteger(0)\nlogical(0)\n[1]   1 -10  NA\n[1]   1 -10  NA\n[1]  TRUE FALSE    NA\n");
        assert!(output.stderr.is_empty());
    }
}
