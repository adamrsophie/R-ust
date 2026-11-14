use super::batch_command;

#[test]
fn expression_mode_runs_additional_string_helpers() {
    let output = batch_command()
        .args([
            "--vanilla",
            "-q",
            "-e",
            "nzchar(c(\"\", \"a\", NA)); nzchar(c(\"\", \"a\", NA), keepNA = TRUE); chartr(\"abc\", \"ABC\", c(\"abracadabra\", NA)); strtrim(c(\"abcdef\", \"xy\", NA), 3); strrep(c(\"a\", \"bc\", NA), c(3, 2, 2))",
        ])
        .output()
        .expect("run r_ust_cli");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains("[1] FALSE  TRUE  TRUE\n"));
    assert!(stdout.contains("[1] FALSE  TRUE    NA\n"));
    assert!(stdout.contains("[1] \"ABrACAdABrA\" NA           \n"));
    assert!(stdout.contains("[1] \"abc\" \"xy\"  NA   \n"));
    assert!(stdout.contains("[1] \"aaa\"  \"bcbc\" NA    \n"));
}

#[test]
fn expression_mode_runs_extract_and_dollar_assignment() {
    let output = batch_command()
        .args([
            "--vanilla",
            "-q",
            "-e",
            "x <- c(1, 2, 3); x[x > 1] <- 0; x; y <- list(a = 1); y$b <- 2; y[[1]] <- 10; names(y) <- c(\"a\", \"renamed\"); y$a; y$renamed; names(y)",
        ])
        .output()
        .expect("run r_ust_cli");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains("[1] 1 0 0\n"));
    assert!(stdout.contains("[1] 10\n"));
    assert!(stdout.contains("[1] 2\n"));
    assert!(stdout.contains("[1] \"a\"       \"renamed\"\n"));
}

#[test]
fn expression_mode_runs_factor_helpers() {
    let output = batch_command()
        .args([
            "--vanilla",
            "-q",
            "-e",
            "f <- factor(c(\"b\", \"a\", \"b\", NA)); f; levels(f); as.character(f); as.numeric(f); is.factor(f); class(f); typeof(f); mode(f); nlevels(f); of <- ordered(c(\"low\", \"high\"), levels = c(\"low\", \"high\")); of; is.ordered(of); class(of); df <- data.frame(x = f, y = 1:4); df",
        ])
        .output()
        .expect("run r_ust_cli");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains("[1] b a b <NA>\nLevels: a b\n"));
    assert!(stdout.contains("[1] \"a\" \"b\"\n"));
    assert!(stdout.contains("[1] \"b\" \"a\" \"b\" NA \n"));
    assert!(stdout.contains("[1]  2  1  2 NA\n"));
    assert!(stdout.contains("[1] TRUE\n"));
    assert!(stdout.contains("[1] \"factor\"\n"));
    assert!(stdout.contains("[1] \"integer\"\n"));
    assert!(stdout.contains("[1] \"numeric\"\n"));
    assert!(stdout.contains("[1] 2\n"));
    assert!(stdout.contains("Levels: low < high\n"));
    assert!(stdout.contains("[1] \"ordered\" \"factor\" \n"));
    assert!(stdout.contains("x y\n1 b 1\n2 a 2\n3 b 3\n4 NA 4\n"));
}

#[test]
fn expression_mode_runs_set_helpers() {
    let output = batch_command()
        .args([
            "--vanilla",
            "-q",
            "-e",
            "union(c(1, 2, NA), c(2, 3, NA)); intersect(c(1, 2, NA), c(2, 3, NA)); setdiff(c(1, 2, NA), c(2, NA)); setequal(c(1, 2, NA), c(2, 1, NA, NA)); setequal(c(1, 2), c(1, 2, 3)); union(c(\"a\", \"b\", NA), c(\"b\", \"c\", NA)); intersect(c(\"a\", \"b\", NA), c(\"b\", \"c\", NA)); setdiff(c(\"a\", \"b\", NA), c(\"b\", NA)); anyDuplicated(c(1, 2, 1, NA, NA)); anyDuplicated(c(NA, 1, NA), fromLast = TRUE); ul <- union(list(a = 1), list(b = 2)); length(ul); ul[[2]]",
        ])
        .output()
        .expect("run r_ust_cli");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains("[1]  1  2 NA  3\n"));
    assert!(stdout.contains("[1]  2 NA\n"));
    assert!(stdout.contains("[1] 1\n"));
    assert!(stdout.contains("[1] TRUE\n"));
    assert!(stdout.contains("[1] FALSE\n"));
    assert!(stdout.contains("[1] \"a\" \"b\" NA  \"c\"\n"));
    assert!(stdout.contains("[1] \"b\" NA \n"));
    assert!(stdout.contains("[1] \"a\"\n"));
    assert!(stdout.contains("[1] 3\n"));
    assert!(stdout.contains("[1] 1\n"));
    assert!(stdout.contains("[1] 2\n"));
}

#[test]
fn expression_mode_runs_string_helpers() {
    let output = batch_command()
        .args([
            "--vanilla",
            "-q",
            "-e",
            "paste(\"a\", c(1, 2), sep = \"-\"); paste0(\"x\", 1:2, collapse = \",\"); nchar(c(\"Ab\", NA)); toupper(\"ab\"); tolower(\"XY\"); substr(c(\"abcd\", NA), 2, 3); substring(\"abcdef\", c(1, 3), c(2, 5)); startsWith(c(\"abc\", \"xbc\", NA), \"a\"); endsWith(c(\"abc\", \"abx\", NA), \"c\"); grepl(\"b\", c(\"abc\", \"def\", NA)); grep(\"b\", c(\"abc\", \"def\", \"bcd\")); grep(\"b\", c(\"abc\", \"def\", \"bcd\"), value = TRUE); strsplit(\"abc\", \"\")[[1]]; sub(\"b\", \"X\", c(\"abc\", \"bdb\", NA)); gsub(\"b\", \"X\", c(\"abc\", \"bdb\", NA)); sub(\"B\", \"X\", \"abc\", ignore.case = TRUE); gsub(\".\", \"X\", c(\"abc\", \"def\"), fixed = TRUE); trimws(c(\"  a \", \"\\tb\\n\", NA)); trimws(\"  a  \", which = \"left\"); trimws(\"  a  \", which = \"right\")",
        ])
        .output()
        .expect("run r_ust_cli");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains("[1] \"a-1\" \"a-2\"\n"));
    assert!(stdout.contains("[1] \"x1,x2\"\n"));
    assert!(stdout.contains("[1]  2 NA\n"));
    assert!(stdout.contains("[1] \"AB\"\n"));
    assert!(stdout.contains("[1] \"xy\"\n"));
    assert!(stdout.contains("[1] \"bc\" NA  \n"));
    assert!(stdout.contains("[1] \"ab\"  \"cde\"\n"));
    assert!(stdout.contains("[1]  TRUE FALSE    NA\n"));
    assert!(stdout.contains("[1]  TRUE FALSE FALSE\n"));
    assert!(stdout.contains("[1] 1 3\n"));
    assert!(stdout.contains("[1] \"abc\" \"bcd\"\n"));
    assert!(stdout.contains("[1] \"a\" \"b\" \"c\"\n"));
    assert!(stdout.contains("[1] \"aXc\" \"Xdb\" NA   \n"));
    assert!(stdout.contains("[1] \"aXc\" \"XdX\" NA   \n"));
    assert!(stdout.contains("[1] \"aXc\"\n"));
    assert!(stdout.contains("[1] \"abc\" \"def\"\n"));
    assert!(stdout.contains("[1] \"a\" \"b\" NA \n"));
    assert!(stdout.contains("[1] \"a  \"\n"));
    assert!(stdout.contains("[1] \"  a\"\n"));
}
