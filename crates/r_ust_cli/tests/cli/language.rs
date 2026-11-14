use super::batch_command;

#[test]
fn expression_mode_runs_right_and_super_assignment() {
    let output = batch_command()
        .args([
            "--vanilla",
            "-q",
            "-e",
            "1 + 2 -> x; x; y <<- 4; y; 5 ->> z; z; bump <- function() { x <<- 9; x }; bump(); x",
        ])
        .output()
        .expect("run r_ust_cli");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert_eq!(stdout, "[1] 3\n[1] 4\n[1] 5\n[1] 9\n[1] 9\n");
}

#[test]
fn expression_mode_runs_lazy_closure_arguments() {
    let output = batch_command()
        .args([
            "--vanilla",
            "-q",
            "-e",
            "f <- function(x) 1; f(missing_symbol); g <- function(x = missing_symbol) 1; g(); h <- function(x = 1 + 1) x; h()",
        ])
        .output()
        .expect("run r_ust_cli");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert_eq!(stdout.matches("[1] 1\n").count(), 2);
    assert!(stdout.contains("[1] 2\n"));
}

#[test]
fn expression_mode_runs_missing_argument_checks() {
    let output = batch_command()
        .args([
            "--vanilla",
            "-q",
            "-e",
            "f <- function(x) missing(x); f(); f(1); g <- function(x = 1) missing(x); g(); g(2)",
        ])
        .output()
        .expect("run r_ust_cli");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert_eq!(stdout.matches("[1] TRUE\n").count(), 2);
    assert_eq!(stdout.matches("[1] FALSE\n").count(), 2);
}

#[test]
fn expression_mode_runs_dots_capture_and_forwarding() {
    let output = batch_command()
        .args([
            "--vanilla",
            "-q",
            "-e",
            "sum_all <- function(...) sum(...); wrap <- function(...) sum_all(...); sum_all(1, 2, 3); wrap(2, 3); capture <- function(...) list(...); capture(1, b = 2)",
        ])
        .output()
        .expect("run r_ust_cli");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains("[1] 6\n"));
    assert!(stdout.contains("[1] 5\n"));
    assert!(stdout.contains("[[1]]\n[1] 1"));
    assert!(stdout.contains("$b\n[1] 2"));
}

#[test]
fn expression_mode_runs_loops_and_subsets() {
    let output = batch_command()
        .args([
            "--vanilla",
            "-q",
            "-e",
            "s <- 0; for (i in 1:5) { if (i == 3) next; if (i == 5) break; s <- s + i }; x <- c(10, 20, 30); s; x[2:3]",
        ])
        .output()
        .expect("run r_ust_cli");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains("[1] 7\n"));
    assert!(stdout.contains("[1] 20 30\n"));
}

#[test]
fn expression_mode_runs_subset_assignment_and_vector_helpers() {
    let output = batch_command()
        .args([
            "--vanilla",
            "-q",
            "-e",
            "x <- c(1, 2, 3); x[2] <- 99; x; rep(seq(2, 6, by = 2), times = 2)",
        ])
        .output()
        .expect("run r_ust_cli");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains("[1]  1 99  3\n"));
    assert!(stdout.contains("[1] 2 4 6 2 4 6\n"));
}

#[test]
fn expression_mode_runs_logical_ops_and_predicates() {
    let output = batch_command()
        .args([
            "--vanilla",
            "-q",
            "-e",
            "FALSE && missing_symbol; TRUE || missing_symbol; any(c(FALSE, TRUE)); is.na(c(1, NaN))",
        ])
        .output()
        .expect("run r_ust_cli");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains("[1] FALSE\n[1] TRUE\n"));
    assert!(stdout.contains("[1] TRUE\n"));
    assert!(stdout.contains("[1] FALSE  TRUE\n"));
}

#[test]
fn expression_mode_runs_logical_identity_helpers() {
    let output = batch_command()
        .args([
            "--vanilla",
            "-q",
            "-e",
            "xor(c(TRUE, FALSE, NA), c(FALSE, FALSE, TRUE)); xor(c(TRUE, FALSE), TRUE); isFALSE(FALSE); isFALSE(NA); isTRUE(1); identical(c(1, NA), c(1, NA)); identical(list(a = 1), list(b = 1)); identical(NaN, NaN)",
        ])
        .output()
        .expect("run r_ust_cli");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains("[1]  TRUE FALSE    NA\n"));
    assert!(stdout.contains("[1] FALSE  TRUE\n"));
    assert!(stdout.contains("[1] TRUE\n"));
    assert!(stdout.contains("[1] FALSE\n"));
}

#[test]
fn expression_mode_runs_lists_and_extractors() {
    let output = batch_command()
        .args([
            "--vanilla",
            "-q",
            "-e",
            "x <- list(a = 1, b = c(2, 3)); x; x$a; x[[2]]; names(x); is.list(x); y <- c(list(a = 1), b = 2); names(y); unlist(y); as.list(c(4, 5))[[2]]",
        ])
        .output()
        .expect("run r_ust_cli");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains("$a\n[1] 1"));
    assert!(stdout.contains("[1] 2 3\n"));
    assert!(stdout.contains("[1] \"a\" \"b\"\n"));
    assert!(stdout.contains("[1] TRUE\n"));
    assert!(stdout.contains("[1] 1 2\n"));
    assert!(stdout.contains("[1] 5\n"));
}
