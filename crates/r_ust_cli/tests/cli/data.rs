use super::batch_command;

#[test]
fn expression_mode_runs_data_frame_helpers() {
    let output = batch_command()
        .args([
            "--vanilla",
            "-q",
            "-e",
            "df <- data.frame(x = 1:3, y = c(\"a\", \"b\", \"c\"), z = 10); df; names(df); dim(df); is.data.frame(df); df$x; df[2, \"x\"]; df[, \"y\"]; df$w <- c(TRUE, FALSE, TRUE); df$w; df[[2]] <- c(\"u\", \"v\", \"w\"); df$y",
        ])
        .output()
        .expect("run r_ust_cli");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains("x y z\n1 1 \"a\" 10\n2 2 \"b\" 10\n3 3 \"c\" 10\n"));
    assert!(stdout.contains("[1] \"x\" \"y\" \"z\"\n"));
    assert!(stdout.contains("[1] 3 3\n"));
    assert!(stdout.contains("[1] TRUE\n"));
    assert!(stdout.contains("[1] 1 2 3\n"));
    assert!(stdout.contains("[1] 2\n"));
    assert!(stdout.contains("[1] \"a\" \"b\" \"c\"\n"));
    assert!(stdout.contains("[1]  TRUE FALSE  TRUE\n"));
    assert!(stdout.contains("[1] \"u\" \"v\" \"w\"\n"));
}

#[test]
fn expression_mode_runs_data_mask_helpers() {
    let output = batch_command()
        .args([
            "--vanilla",
            "-q",
            "-e",
            "df <- data.frame(x = 1:3, y = c(\"a\", \"b\", \"c\")); with(df, x + 1); subset(df, x > 1); subset(df, x > 1, select = y); transform(df, z = x + 10, y = toupper(y))",
        ])
        .output()
        .expect("run r_ust_cli");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains("[1] 2 3 4\n"));
    assert!(stdout.contains("x y\n1 2 \"b\"\n2 3 \"c\"\n"));
    assert!(stdout.contains("y\n1 \"b\"\n2 \"c\"\n"));
    assert!(stdout.contains("x y z\n1 1 \"A\" 11\n2 2 \"B\" 12\n3 3 \"C\" 13\n"));
}

#[test]
fn expression_mode_runs_missing_data_helpers() {
    let output = batch_command()
        .args([
            "--vanilla",
            "-q",
            "-e",
            "ifelse(c(TRUE, FALSE, NA), 1:3, 10:12); df <- data.frame(x = c(1, NA, 3), y = c(\"a\", \"b\", NA)); complete.cases(df); na.omit(df); na.omit(c(1, NA, 3)); na.omit(factor(c(\"a\", NA, \"b\")))",
        ])
        .output()
        .expect("run r_ust_cli");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains("[1]  1 11 NA\n"));
    assert!(stdout.contains("[1]  TRUE FALSE FALSE\n"));
    assert!(stdout.contains("x y\n1 1 \"a\"\n"));
    assert!(stdout.contains("[1] 1 3\n"));
    assert!(stdout.contains("[1] a b\nLevels: a b\n"));
}

#[test]
fn expression_mode_runs_split_and_table_helpers() {
    let output = batch_command()
        .args([
            "--vanilla",
            "-q",
            "-e",
            "df <- data.frame(x = 1:4, g = factor(c(\"a\", \"b\", \"a\", \"b\"))); split(1:4, df$g); split(df, df$g); table(df$g); table(c(\"b\", \"a\", \"b\", NA), useNA = \"ifany\"); table(df$g, c(\"x\", \"x\", \"y\", \"y\"))",
        ])
        .output()
        .expect("run r_ust_cli");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains("$a\n[1] 1 3\n"));
    assert!(stdout.contains("$b\n[1] 2 4\n"));
    assert!(stdout.contains("$a\n  x g\n1 1 a\n2 3 a\n"));
    assert!(stdout.contains("$a\n[1] 2\n"));
    assert!(stdout.contains("$<NA>\n[1] 1\n"));
    assert!(stdout.contains("[1,] 1 1\n[2,] 1 1\n"));
}

#[test]
fn expression_mode_runs_lapply_and_sapply() {
    let output = batch_command()
        .args([
            "--vanilla",
            "-q",
            "-e",
            "lapply(1:3, function(x) x * 2); sapply(1:3, function(x) x * 2); sapply(1:3, function(x) c(x, x + 10)); sapply(1:3, sqrt); sapply(1:3, function(x, inc) x + inc, inc = 10); df <- data.frame(x = 1:3, y = 4:6); sapply(df, mean)",
        ])
        .output()
        .expect("run r_ust_cli");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains("[[2]]\n[1] 4\n"));
    assert!(stdout.contains("[1] 2 4 6\n"));
    assert!(stdout.contains("[1,] 1 2 3\n[2,] 11 12 13\n"));
    assert!(stdout.contains("[1]              1 1.414213562373 1.732050807569\n"));
    assert!(stdout.contains("[1] 11 12 13\n"));
    assert!(stdout.contains("[1] 2 5\n"));
}

#[test]
fn expression_mode_runs_bind_helpers() {
    let output = batch_command()
        .args([
            "--vanilla",
            "-q",
            "-e",
            "cbind(1:2, 3:4); rbind(1:2, 3:4); cbind(data.frame(x = 1:2), y = 3:4); rbind(data.frame(x = 1:2, y = 3:4), data.frame(x = 5:6, y = 7:8))",
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
    assert!(stdout.contains("[1,] 1 2\n[2,] 3 4\n"));
    assert!(stdout.contains("x y\n1 1 3\n2 2 4\n"));
    assert!(stdout.contains("x y\n1 1 3\n2 2 4\n3 5 7\n4 6 8\n"));
}
