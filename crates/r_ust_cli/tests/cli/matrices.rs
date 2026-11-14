use super::batch_command;

#[test]
fn expression_mode_runs_matrix_apply() {
    let output = batch_command()
        .args([
            "--vanilla",
            "-q",
            "-e",
            "m <- matrix(1:6, nrow = 2); apply(m, 1, sum); apply(m, 2, mean); apply(m, 1, function(x) c(x[1], sum(x))); apply(m, 2, function(x, inc) mean(x) + inc, inc = 10)",
        ])
        .output()
        .expect("run r_ust_cli");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains("[1]  9 12\n"));
    assert!(stdout.contains("[1] 1.5 3.5 5.5\n"));
    assert!(stdout.contains("[1,] 1 2\n[2,] 9 12\n"));
    assert!(stdout.contains("[1] 11.5 13.5 15.5\n"));
}

#[test]
fn expression_mode_runs_matrix_helpers() {
    let output = batch_command()
        .args([
            "--vanilla",
            "-q",
            "-e",
            "m <- matrix(1:4, nrow = 2); m; dim(m); nrow(m); ncol(m); m[1, 2]; m[, 2]; t(m); rowSums(m); colMeans(m); m[1, 2] <- 99; m[1, 2]; matrix(1:4, nrow = 2, byrow = TRUE)",
        ])
        .output()
        .expect("run r_ust_cli");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains("[1,] 1 3\n[2,] 2 4\n"));
    assert!(stdout.contains("[1] 2 2\n"));
    assert!(stdout.contains("[1] 2\n[1] 2\n"));
    assert!(stdout.contains("[1] 3\n"));
    assert!(stdout.contains("[1,] 1 2\n[2,] 3 4\n"));
    assert!(stdout.contains("[1] 4 6\n"));
    assert!(stdout.contains("[1] 1.5 3.5\n"));
    assert!(stdout.contains("[1] 99\n"));
    assert!(stdout.contains("[1,] 1 2\n[2,] 3 4\n"));
}

#[test]
fn expression_mode_runs_matrix_index_generators() {
    let output = batch_command()
        .args([
            "--vanilla",
            "-q",
            "-e",
            "m <- matrix(1:6, nrow = 2); diag(m); diag(c(5, 6)); diag(3); row(m); col(m); lower.tri(matrix(1:9, nrow = 3)); upper.tri(matrix(1:9, nrow = 3), diag = TRUE)",
        ])
        .output()
        .expect("run r_ust_cli");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains("[1] 1 4\n"));
    assert!(stdout.contains("[1,] 5 0\n[2,] 0 6\n"));
    assert!(stdout.contains("[1,] 1 0 0\n[2,] 0 1 0\n[3,] 0 0 1\n"));
    assert!(stdout.contains("[1,] 1 1 1\n[2,] 2 2 2\n"));
    assert!(stdout.contains("[1,] 1 2 3\n[2,] 1 2 3\n"));
    assert!(
        stdout.contains("[1,] FALSE FALSE FALSE\n[2,] TRUE FALSE FALSE\n[3,] TRUE TRUE FALSE\n")
    );
    assert!(stdout.contains("[1,] TRUE TRUE TRUE\n[2,] FALSE TRUE TRUE\n[3,] FALSE FALSE TRUE\n"));
}

#[test]
fn expression_mode_runs_matrix_crossprod_helpers() {
    let output = batch_command()
        .args([
            "--vanilla",
            "-q",
            "-e",
            "m <- matrix(1:6, nrow = 2); crossprod(m); crossprod(m, matrix(1:4, nrow = 2)); tcrossprod(m); crossprod(1:3); tcrossprod(1:3); crossprod(1:3, 4:6); tcrossprod(1:3, 4:6)",
        ])
        .output()
        .expect("run r_ust_cli");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains("[1,] 5 11 17\n[2,] 11 25 39\n[3,] 17 39 61\n"));
    assert!(stdout.contains("[1,] 5 11\n[2,] 11 25\n[3,] 17 39\n"));
    assert!(stdout.contains("[1,] 35 44\n[2,] 44 56\n"));
    assert!(stdout.contains("[1,] 14\n"));
    assert!(stdout.contains("[1,] 1 2 3\n[2,] 2 4 6\n[3,] 3 6 9\n"));
    assert!(stdout.contains("[1,] 32\n"));
    assert!(stdout.contains("[1,] 4 5 6\n[2,] 8 10 12\n[3,] 12 15 18\n"));
}
