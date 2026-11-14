use super::batch_command;

#[test]
fn expression_mode_runs_structural_vector_helpers() {
    let output = batch_command()
        .args([
            "--vanilla",
            "-q",
            "-e",
            "seq.int(2, 6, by = 2); rep.int(c(1, 2), 2); rep_len(c(\"a\", \"b\"), 5); append(1:3, 9); append(1:3, 9, after = 1); lst <- append(list(a = 1), list(b = 2)); names(lst); lst$b; mixed <- append(1:2, list(3)); mixed[[3]]; replace(c(1, 2, 3), c(1, 3), c(9, 8)); replace(c(\"a\", \"b\", \"c\"), c(TRUE, FALSE, TRUE), \"z\"); rl <- replace(list(a = 1, b = 2), 2, list(z = 9)); names(rl); rl$b; lengths(list(a = 1:3, b = list(1, 2), c = NULL)); lengths(data.frame(x = 1:3, y = c(\"a\", \"b\", \"c\"))); lengths(c(TRUE, FALSE, NA))",
        ])
        .output()
        .expect("run r_ust_cli");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains("[1] 2 4 6\n"));
    assert!(stdout.contains("[1] 1 2 1 2\n"));
    assert!(stdout.contains("[1] \"a\" \"b\" \"a\" \"b\" \"a\"\n"));
    assert!(stdout.contains("[1] 1 2 3 9\n"));
    assert!(stdout.contains("[1] 1 9 2 3\n"));
    assert!(stdout.contains("[1] \"a\" \"b\"\n"));
    assert!(stdout.contains("[1] 9 2 8\n"));
    assert!(stdout.contains("[1] \"z\" \"b\" \"z\"\n"));
    assert!(stdout.contains("[1] 3 2 0\n"));
    assert!(stdout.contains("[1] 3 3\n"));
    assert!(stdout.contains("[1] 1 1 1\n"));
}

#[test]
fn expression_mode_runs_statistical_vector_helpers() {
    let output = batch_command()
        .args([
            "--vanilla",
            "-q",
            "-e",
            "range(c(3, 1, 4, NA)); range(c(3, 1, 4, NA), na.rm = TRUE); median(c(3, 1, 4)); median(c(3, 1, 4, 2)); var(c(1, 2, 3)); sd(c(1, 2, 3)); diff(c(1, 4, 9)); diff(c(1, 4, 9, 16), lag = 2); diff(c(1, 4, 9, 16), differences = 2); cumsum(c(1, 2, NA, 4)); cumprod(c(1, 2, NA, 4)); cummin(c(3, 1, NA, 0)); cummax(c(3, 1, NA, 5)); pmin(c(1, 5, NA), c(2, 3, 4)); pmin(c(1, 5, NA), c(2, 3, 4), na.rm = TRUE); pmax(c(1, 5, NA), c(2, 3, 4)); pmax(c(1, 5, NA), c(2, 3, 4), na.rm = TRUE)",
        ])
        .output()
        .expect("run r_ust_cli");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains("[1] NA NA\n"));
    assert!(stdout.contains("[1] 1 4\n"));
    assert!(stdout.contains("[1] 3\n"));
    assert!(stdout.contains("[1] 2.5\n"));
    assert!(stdout.contains("[1] 1\n"));
    assert!(stdout.contains("[1] 3 5\n"));
    assert!(stdout.contains("[1]  8 12\n"));
    assert!(stdout.contains("[1] 2 2\n"));
    assert!(stdout.contains("[1]  1  3 NA NA\n"));
    assert!(stdout.contains("[1]  1  2 NA NA\n"));
    assert!(stdout.contains("[1]  3  1 NA NA\n"));
    assert!(stdout.contains("[1]  3  3 NA NA\n"));
    assert!(stdout.contains("[1]  1  3 NA\n"));
    assert!(stdout.contains("[1] 1 3 4\n"));
    assert!(stdout.contains("[1]  2  5 NA\n"));
    assert!(stdout.contains("[1] 2 5 4\n"));
}

#[test]
fn expression_mode_runs_logical_subsetting_and_match() {
    let output = batch_command()
        .args([
            "--vanilla",
            "-q",
            "-e",
            "x <- c(10, 20, 30, 40); x[x > 20]; which(x >= 30); c(\"b\", \"z\") %in% c(\"a\", \"b\"); match(c(\"b\", \"z\"), c(\"a\", \"b\"))",
        ])
        .output()
        .expect("run r_ust_cli");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains("[1] 30 40\n"));
    assert!(stdout.contains("[1] 3 4\n"));
    assert!(stdout.contains("[1]  TRUE FALSE\n"));
    assert!(stdout.contains("[1]  2 NA\n"));
}

#[test]
fn expression_mode_runs_vector_order_helpers() {
    let output = batch_command()
        .args([
            "--vanilla",
            "-q",
            "-e",
            "head(1:5, 2); tail(1:5, 2); rev(1:3); sort(c(3, 1, 2)); order(c(3, 1, 2)); rank(c(30, 10, 20)); unique(c(1, 2, 1, NA, NA)); duplicated(c(1, 2, 1, NA, NA)); df <- data.frame(x = c(3, 1, 2), y = c(\"c\", \"a\", \"b\")); df[order(df$x), \"y\"]",
        ])
        .output()
        .expect("run r_ust_cli");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains("[1] 1 2\n"));
    assert!(stdout.contains("[1] 4 5\n"));
    assert!(stdout.contains("[1] 3 2 1\n"));
    assert!(stdout.contains("[1] 1 2 3\n"));
    assert!(stdout.contains("[1] 2 3 1\n"));
    assert!(stdout.contains("[1] 3 1 2\n"));
    assert!(stdout.contains("[1]  1  2 NA\n"));
    assert!(stdout.contains("[1] FALSE FALSE  TRUE FALSE  TRUE\n"));
    assert!(stdout.contains("[1] \"a\" \"b\" \"c\"\n"));
}

#[test]
fn expression_mode_runs_position_sortedness_and_tabulate_helpers() {
    let output = batch_command()
        .args([
            "--vanilla",
            "-q",
            "-e",
            "which.min(c(3, NA, 2, 2)); which.max(c(3, NA, 4, 4)); which.max(c(FALSE, TRUE, NA)); is.unsorted(c(1, 2, 2)); is.unsorted(c(1, 2, 2), strictly = TRUE); is.unsorted(c(1, NA, 2)); is.unsorted(c(2, NA, 1), na.rm = TRUE); tabulate(c(1, 2, 2, 0, NA, 3.9), nbins = 4); tabulate(c(2, 2), nbins = 4); tabulate(c(1, 3, 3))",
        ])
        .output()
        .expect("run r_ust_cli");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains("[1] 3\n"));
    assert!(stdout.contains("[1] 2\n"));
    assert!(stdout.contains("[1] FALSE\n"));
    assert!(stdout.contains("[1] TRUE\n"));
    assert!(stdout.contains("[1] NA\n"));
    assert!(stdout.contains("[1] 1 2 1 0\n"));
    assert!(stdout.contains("[1] 0 2 0 0\n"));
    assert!(stdout.contains("[1] 1 0 2\n"));
}
