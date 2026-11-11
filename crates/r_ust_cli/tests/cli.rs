use std::process::Command;

#[test]
fn expression_mode_runs_arithmetic_and_rmath() {
    let output = Command::new(env!("CARGO_BIN_EXE_r_ust_cli"))
        .args(["--vanilla", "-q", "-e", "x <- 1 + 2 * 3; x; dnorm(0)"])
        .output()
        .expect("run r_ust_cli");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert_eq!(stdout, "[1] 7\n[1] 0.398942280401\n");
}

#[test]
fn expression_mode_runs_right_and_super_assignment() {
    let output = Command::new(env!("CARGO_BIN_EXE_r_ust_cli"))
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
fn expression_mode_runs_user_functions_and_return() {
    let output = Command::new(env!("CARGO_BIN_EXE_r_ust_cli"))
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
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains("function(x)\n"));
    assert!(stdout.contains("[1] 3\n[1] 4\n"));
}

#[test]
fn expression_mode_runs_lazy_closure_arguments() {
    let output = Command::new(env!("CARGO_BIN_EXE_r_ust_cli"))
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
    let output = Command::new(env!("CARGO_BIN_EXE_r_ust_cli"))
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
    let output = Command::new(env!("CARGO_BIN_EXE_r_ust_cli"))
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
    let output = Command::new(env!("CARGO_BIN_EXE_r_ust_cli"))
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
    let output = Command::new(env!("CARGO_BIN_EXE_r_ust_cli"))
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
    assert!(stdout.contains("[1] 1 99 3\n"));
    assert!(stdout.contains("[1] 2 4 6 2 4 6\n"));
}

#[test]
fn expression_mode_runs_logical_ops_and_predicates() {
    let output = Command::new(env!("CARGO_BIN_EXE_r_ust_cli"))
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
    assert!(stdout.contains("[1] FALSE TRUE\n"));
}

#[test]
fn expression_mode_runs_logical_identity_helpers() {
    let output = Command::new(env!("CARGO_BIN_EXE_r_ust_cli"))
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
    assert!(stdout.contains("[1] TRUE FALSE NA\n"));
    assert!(stdout.contains("[1] FALSE TRUE\n"));
    assert!(stdout.contains("[1] TRUE\n"));
    assert!(stdout.contains("[1] FALSE\n"));
}

#[test]
fn expression_mode_runs_uniform_sampling_helpers() {
    let output = Command::new(env!("CARGO_BIN_EXE_r_ust_cli"))
        .args([
            "--vanilla",
            "-q",
            "-e",
            "set.seed(42); sample.int(5); set.seed(42); sample.int(5, 3); set.seed(42); sample.int(3, 6, replace = TRUE); set.seed(42); sample(5); set.seed(42); sample(c(\"a\", \"b\", \"c\"), 2); set.seed(42); sampled <- sample(list(a = 1, b = 2), 2); names(sampled); sampled[[1]]; sampled[[2]]",
        ])
        .output()
        .expect("run r_ust_cli");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert_eq!(stdout.matches("[1] 3 1 5 4 2\n").count(), 2);
    assert!(stdout.contains("[1] 3 1 5\n"));
    assert!(stdout.contains("[1] 3 2 3 3 1 1\n"));
    assert!(stdout.contains("[1] \"c\" \"a\"\n"));
    assert!(stdout.contains("[1] \"a\" \"b\"\n"));
    assert!(stdout.contains("[1] 1\n"));
    assert!(stdout.contains("[1] 2\n"));
}

#[test]
fn expression_mode_runs_additional_random_distribution_helpers() {
    let output = Command::new(env!("CARGO_BIN_EXE_r_ust_cli"))
        .args([
            "--vanilla",
            "-q",
            "-e",
            "set.seed(7); rexp(2, rate = 2); set.seed(7); rcauchy(2, location = 1, scale = 2); set.seed(7); rlogis(2, location = 1, scale = 2); set.seed(7); rweibull(2, shape = 2, scale = 3); set.seed(7); rlnorm(2, meanlog = 1, sdlog = 0.5); set.seed(7); rpois(2, lambda = 4); set.seed(7); rgeom(2, prob = 0.25); set.seed(7); rbinom(2, size = 10, prob = 0.4); set.seed(7); rgamma(2, shape = 3, rate = 2); set.seed(7); rchisq(2, df = 4); set.seed(7); rbeta(2, shape1 = 2, shape2 = 3); set.seed(7); rf(2, df1 = 4, df2 = 6); set.seed(7); rt(2, df = 5); set.seed(7); rnbinom(2, size = 5, prob = 0.4); set.seed(7); rnbinom(2, size = 5, mu = 4); set.seed(7); rnchisq(2, df = 4, ncp = 1); set.seed(7); rhyper(3, m = 5, n = 7, k = 4); set.seed(7); rsignrank(3, n = 4); set.seed(7); rwilcox(3, m = 3, n = 4); set.seed(7); rhyper(nn = 3, m = 5, n = 7, k = 4)",
        ])
        .output()
        .expect("run r_ust_cli");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains("[1] 0.018245990155 0.691220217094\n"));
    assert!(stdout.contains("[1] 0.675034031277 1.333839266687\n"));
    assert!(stdout.contains("[1] 6.835960082702 -4.780133104908\n"));
    assert!(stdout.contains("[1] 0.688256497741 5.147557532988\n"));
    assert!(stdout.contains("[1] 6.149202253105 3.471663233511\n"));
    assert!(stdout.contains("[1] 3 2\n"));
    assert!(stdout.contains("[1] 0 7\n"));
    assert!(stdout.contains("[1] 5 5\n"));
    assert!(stdout.contains("[1] 3.159988069084 1.17965232797\n"));
    assert!(stdout.contains("[1] 9.57552211355 2.852550537173\n"));
    assert!(stdout.contains("[1] 0.669891847819 0.279765194257\n"));
    assert!(stdout.contains("[1] 3.043965334059 0.582654140066\n"));
    assert!(stdout.contains("[1] 1.498005549196 0.067031526736\n"));
    assert!(stdout.contains("[1] 9 4\n"));
    assert!(stdout.contains("[1] 3 4\n"));
    assert!(stdout.contains("[1] 7.0961758966 0.521138258407\n"));
    assert_eq!(stdout.matches("[1] 2 3 1\n").count(), 2);
    assert!(stdout.contains("[1] 6 8 2\n"));
    assert!(stdout.contains("[1] 8 3 3\n"));
}

#[test]
fn expression_mode_runs_lists_and_extractors() {
    let output = Command::new(env!("CARGO_BIN_EXE_r_ust_cli"))
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

#[test]
fn expression_mode_runs_data_frame_helpers() {
    let output = Command::new(env!("CARGO_BIN_EXE_r_ust_cli"))
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
    assert!(stdout.contains("[1] TRUE FALSE TRUE\n"));
    assert!(stdout.contains("[1] \"u\" \"v\" \"w\"\n"));
}

#[test]
fn expression_mode_runs_factor_helpers() {
    let output = Command::new(env!("CARGO_BIN_EXE_r_ust_cli"))
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
    assert!(stdout.contains("[1] \"b\" \"a\" \"b\" NA\n"));
    assert!(stdout.contains("[1] 2 1 2 NaN\n"));
    assert!(stdout.contains("[1] TRUE\n"));
    assert!(stdout.contains("[1] \"factor\"\n"));
    assert!(stdout.contains("[1] \"integer\"\n"));
    assert!(stdout.contains("[1] \"numeric\"\n"));
    assert!(stdout.contains("[1] 2\n"));
    assert!(stdout.contains("Levels: low < high\n"));
    assert!(stdout.contains("[1] \"ordered\" \"factor\"\n"));
    assert!(stdout.contains("x y\n1 b 1\n2 a 2\n3 b 3\n4 NA 4\n"));
}

#[test]
fn expression_mode_runs_coercion_and_type_predicates() {
    let output = Command::new(env!("CARGO_BIN_EXE_r_ust_cli"))
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
    assert!(stdout.contains("[1] 1 -1 NA NA\n"));
    assert!(stdout.contains("[1] 1 NaN\n"));
    assert!(stdout.contains("[1] 1 0 NaN\n"));
    assert!(stdout.contains("[1] TRUE\n"));
    assert!(stdout.contains("[1] FALSE\n"));
}

#[test]
fn expression_mode_runs_structural_vector_helpers() {
    let output = Command::new(env!("CARGO_BIN_EXE_r_ust_cli"))
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
fn expression_mode_runs_base_numeric_helpers() {
    let output = Command::new(env!("CARGO_BIN_EXE_r_ust_cli"))
        .args([
            "--vanilla",
            "-q",
            "-e",
            "is.nan(c(0, NaN, Inf)); is.finite(c(0, NaN, Inf, -Inf)); is.infinite(c(0, NaN, Inf, -Inf)); floor(c(-1.2, 1.8, NaN)); ceiling(c(-1.2, 1.2, NaN)); trunc(c(-1.8, 1.8, NaN)); sign(c(-2, 0, 3, NaN)); log10(c(1, 10, 100)); log2(c(1, 2, 8)); sin(c(0, 1.5707963267948966)); cos(c(0, 3.141592653589793)); round(c(1.234, 5.678), digits = 1); round(1234, digits = -2)",
        ])
        .output()
        .expect("run r_ust_cli");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains("[1] FALSE TRUE FALSE\n"));
    assert!(stdout.contains("[1] TRUE FALSE FALSE FALSE\n"));
    assert!(stdout.contains("[1] FALSE FALSE TRUE TRUE\n"));
    assert!(stdout.contains("[1] -2 1 NaN\n"));
    assert!(stdout.contains("[1] -1 2 NaN\n"));
    assert!(stdout.contains("[1] -1 1 NaN\n"));
    assert!(stdout.contains("[1] -1 0 1 NaN\n"));
    assert!(stdout.contains("[1] 0 1 2\n"));
    assert!(stdout.contains("[1] 0 1 3\n"));
    assert!(stdout.contains("[1] 0 1\n"));
    assert!(stdout.contains("[1] 1 -1\n"));
    assert!(stdout.contains("[1] 1.2 5.7\n"));
    assert!(stdout.contains("[1] 1200\n"));
}

#[test]
fn expression_mode_runs_special_numeric_helpers() {
    let output = Command::new(env!("CARGO_BIN_EXE_r_ust_cli"))
        .args([
            "--vanilla",
            "-q",
            "-e",
            "logb(c(8, 16), base = 2); atan2(c(1, 0), c(0, -1)); sinh(c(0, 1)); cosh(0); tanh(c(0, 1)); asinh(1); acosh(1); atanh(0.5); signif(c(1.2345, 12345), digits = 3); factorial(c(0, 5)); lfactorial(5); lbeta(2, 3); lchoose(5, 2); digamma(1); trigamma(1); psigamma(1, deriv = 1)",
        ])
        .output()
        .expect("run r_ust_cli");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains("[1] 3 4\n"));
    assert!(stdout.contains("[1] 1.570796326795 3.14159265359\n"));
    assert!(stdout.contains("[1] 0 1.175201193644\n"));
    assert!(stdout.contains("[1] 1\n"));
    assert!(stdout.contains("[1] 0 0.761594155956\n"));
    assert!(stdout.contains("[1] 0.88137358702\n"));
    assert!(stdout.contains("[1] 0.549306144334\n"));
    assert!(stdout.contains("[1] 1.23 12300\n"));
    assert!(stdout.contains("[1] 1 120\n"));
    assert!(stdout.contains("[1] 4.787491742782\n"));
    assert!(stdout.contains("[1] -2.484906649788\n"));
    assert!(stdout.contains("[1] 2.302585092994\n"));
    assert!(stdout.contains("[1] -0.577215664902\n"));
    assert!(stdout.contains("[1] 1.644934066848"));
}

#[test]
fn expression_mode_runs_additional_distribution_families() {
    let output = Command::new(env!("CARGO_BIN_EXE_r_ust_cli"))
        .args([
            "--vanilla",
            "-q",
            "-e",
            "dgeom(2, prob = 0.25); pgeom(2, prob = 0.25); qgeom(0.5, prob = 0.25); dlnorm(2, meanlog = 0, sdlog = 1); plnorm(2, meanlog = 0, sdlog = 1); qlnorm(0.5, meanlog = 0, sdlog = 1); dweibull(2, shape = 3, scale = 4); pweibull(2, shape = 3, scale = 4); qweibull(0.5, shape = 3, scale = 4); pgeom(c(2, 3), prob = 0.25, lower.tail = FALSE, log.p = TRUE)",
        ])
        .output()
        .expect("run r_ust_cli");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains("[1] 0.140625\n"));
    assert!(stdout.contains("[1] 0.578125\n"));
    assert!(stdout.contains("[1] 2\n"));
    assert!(stdout.contains("[1] 0.156874019279\n"));
    assert!(stdout.contains("[1] 0.755891404214\n"));
    assert!(stdout.contains("[1] 1\n"));
    assert!(stdout.contains("[1] 0.165468169235\n"));
    assert!(stdout.contains("[1] 0.117503097415\n"));
    assert!(stdout.contains("[1] 3.539988178002\n"));
    assert!(stdout.contains("[1] -0.863046217355 -1.150728289807\n"));
}

#[test]
fn expression_mode_runs_continuous_distribution_families() {
    let output = Command::new(env!("CARGO_BIN_EXE_r_ust_cli"))
        .args([
            "--vanilla",
            "-q",
            "-e",
            "dbeta(0.25, shape1 = 2, shape2 = 3); pbeta(0.25, shape1 = 2, shape2 = 3); qbeta(0.5, shape1 = 2, shape2 = 3); dchisq(2, df = 4); pchisq(2, df = 4); qchisq(0.5, df = 4); dt(1, df = 5); pt(1, df = 5); qt(0.75, df = 5); df(2, df1 = 4, df2 = 6); pf(2, df1 = 4, df2 = 6); qf(0.5, df1 = 4, df2 = 6)",
        ])
        .output()
        .expect("run r_ust_cli");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains("[1] 1.6875\n"));
    assert!(stdout.contains("[1] 0.26171875\n"));
    assert!(stdout.contains("[1] 0.385727568132\n"));
    assert!(stdout.contains("[1] 0.183939720586\n"));
    assert!(stdout.contains("[1] 0.264241117657\n"));
    assert!(stdout.contains("[1] 3.356693980033\n"));
    assert!(stdout.contains("[1] 0.219679797351\n"));
    assert!(stdout.contains("[1] 0.818391266175\n"));
    assert!(stdout.contains("[1] 0.7266868438\n"));
    assert!(stdout.contains("[1] 0.154221455346\n"));
    assert!(stdout.contains("[1] 0.786339025406\n"));
    assert!(stdout.contains("[1] 0.941913265486\n"));
}

#[test]
fn expression_mode_runs_gamma_and_hypergeometric_distribution_families() {
    let output = Command::new(env!("CARGO_BIN_EXE_r_ust_cli"))
        .args([
            "--vanilla",
            "-q",
            "-e",
            "pgamma(2, shape = 3, scale = 4); qgamma(0.5, shape = 3, scale = 4); dhyper(2, m = 5, n = 7, k = 4); phyper(2, m = 5, n = 7, k = 4); qhyper(0.5, m = 5, n = 7, k = 4)",
        ])
        .output()
        .expect("run r_ust_cli");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains("[1] 0.014387677967\n"));
    assert!(stdout.contains("[1] 10.696241254894\n"));
    assert!(stdout.contains("[1] 0.424242424242\n"));
    assert!(stdout.contains("[1] 0.848484848485\n"));
    assert!(stdout.contains("[1] 2\n"));
}

#[test]
fn expression_mode_runs_negative_binomial_and_rank_distribution_families() {
    let output = Command::new(env!("CARGO_BIN_EXE_r_ust_cli"))
        .args([
            "--vanilla",
            "-q",
            "-e",
            "dnbinom(3, size = 5, prob = 0.4); pnbinom(3, size = 5, prob = 0.4); qnbinom(0.5, size = 5, prob = 0.4); dnbinom(3, size = 5, mu = 4); pnbinom(3, size = 5, mu = 4); qnbinom(0.5, size = 5, mu = 4); dsignrank(3, n = 4); psignrank(3, n = 4); qsignrank(0.5, n = 4); dwilcox(3, m = 3, n = 4); pwilcox(3, m = 3, n = 4); qwilcox(0.5, m = 3, n = 4)",
        ])
        .output()
        .expect("run r_ust_cli");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains("[1] 0.0774144\n"));
    assert!(stdout.contains("[1] 0.1736704\n"));
    assert!(stdout.contains("[1] 7\n"));
    assert!(stdout.contains("[1] 0.162614011878\n"));
    assert!(stdout.contains("[1] 0.489947306323\n"));
    assert!(stdout.contains("[1] 4\n"));
    assert!(stdout.contains("[1] 0.125\n"));
    assert!(stdout.contains("[1] 0.3125\n"));
    assert!(stdout.contains("[1] 5\n"));
    assert!(stdout.contains("[1] 0.085714285714\n"));
    assert!(stdout.contains("[1] 0.2\n"));
    assert!(stdout.contains("[1] 6\n"));
}

#[test]
fn expression_mode_runs_noncentral_distribution_forms() {
    let output = Command::new(env!("CARGO_BIN_EXE_r_ust_cli"))
        .args([
            "--vanilla",
            "-q",
            "-e",
            "dbeta(0.25, shape1 = 2, shape2 = 3, ncp = 1); pbeta(0.25, shape1 = 2, shape2 = 3, ncp = 1); qbeta(0.5, shape1 = 2, shape2 = 3, ncp = 1); dchisq(2, df = 4, ncp = 1); pchisq(2, df = 4, ncp = 1); qchisq(0.5, df = 4, ncp = 1); dt(1, df = 5, ncp = 0.75); pt(1, df = 5, ncp = 0.75); qt(0.75, df = 5, ncp = 0.75); df(2, df1 = 4, df2 = 6, ncp = 1); pf(2, df1 = 4, df2 = 6, ncp = 1); qf(0.5, df1 = 4, df2 = 6, ncp = 1)",
        ])
        .output()
        .expect("run r_ust_cli");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains("[1] 1.386418608787\n"));
    assert!(stdout.contains("[1] 0.19315328883\n"));
    assert!(stdout.contains("[1] 0.441606391854\n"));
    assert!(stdout.contains("[1] 0.141879929236\n"));
    assert!(stdout.contains("[1] 0.186109779331\n"));
    assert!(stdout.contains("[1] 4.234893032395\n"));
    assert!(stdout.contains("[1] 0.348396866146\n"));
    assert!(stdout.contains("[1] 0.575809655591\n"));
    assert!(stdout.contains("[1] 1.579344079944\n"));
    assert!(stdout.contains("[1] 0.18302282189\n"));
    assert!(stdout.contains("[1] 0.710406862259\n"));
    assert!(stdout.contains("[1] 1.186277167425\n"));
}

#[test]
fn expression_mode_runs_statistical_vector_helpers() {
    let output = Command::new(env!("CARGO_BIN_EXE_r_ust_cli"))
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
    assert!(stdout.contains("[1] NaN NaN\n"));
    assert!(stdout.contains("[1] 1 4\n"));
    assert!(stdout.contains("[1] 3\n"));
    assert!(stdout.contains("[1] 2.5\n"));
    assert!(stdout.contains("[1] 1\n"));
    assert!(stdout.contains("[1] 3 5\n"));
    assert!(stdout.contains("[1] 8 12\n"));
    assert!(stdout.contains("[1] 2 2\n"));
    assert!(stdout.contains("[1] 1 3 NaN NaN\n"));
    assert!(stdout.contains("[1] 1 2 NaN NaN\n"));
    assert!(stdout.contains("[1] 3 1 NaN NaN\n"));
    assert!(stdout.contains("[1] 3 3 NaN NaN\n"));
    assert!(stdout.contains("[1] 1 3 NaN\n"));
    assert!(stdout.contains("[1] 1 3 4\n"));
    assert!(stdout.contains("[1] 2 5 NaN\n"));
    assert!(stdout.contains("[1] 2 5 4\n"));
}

#[test]
fn expression_mode_runs_data_mask_helpers() {
    let output = Command::new(env!("CARGO_BIN_EXE_r_ust_cli"))
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
    let output = Command::new(env!("CARGO_BIN_EXE_r_ust_cli"))
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
    assert!(stdout.contains("[1] 1 11 NaN\n"));
    assert!(stdout.contains("[1] TRUE FALSE FALSE\n"));
    assert!(stdout.contains("x y\n1 1 \"a\"\n"));
    assert!(stdout.contains("[1] 1 3\n"));
    assert!(stdout.contains("[1] a b\nLevels: a b\n"));
}

#[test]
fn expression_mode_runs_split_and_table_helpers() {
    let output = Command::new(env!("CARGO_BIN_EXE_r_ust_cli"))
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
    let output = Command::new(env!("CARGO_BIN_EXE_r_ust_cli"))
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
    assert!(stdout.contains("[1] 1 1.414213562373 1.732050807569\n"));
    assert!(stdout.contains("[1] 11 12 13\n"));
    assert!(stdout.contains("[1] 2 5\n"));
}

#[test]
fn expression_mode_runs_matrix_apply() {
    let output = Command::new(env!("CARGO_BIN_EXE_r_ust_cli"))
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
    assert!(stdout.contains("[1] 9 12\n"));
    assert!(stdout.contains("[1] 1.5 3.5 5.5\n"));
    assert!(stdout.contains("[1,] 1 2\n[2,] 9 12\n"));
    assert!(stdout.contains("[1] 11.5 13.5 15.5\n"));
}

#[test]
fn expression_mode_runs_bind_helpers() {
    let output = Command::new(env!("CARGO_BIN_EXE_r_ust_cli"))
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

#[test]
fn expression_mode_runs_logical_subsetting_and_match() {
    let output = Command::new(env!("CARGO_BIN_EXE_r_ust_cli"))
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
    assert!(stdout.contains("[1] TRUE FALSE\n"));
    assert!(stdout.contains("[1] 2 NA\n"));
}

#[test]
fn expression_mode_runs_string_helpers() {
    let output = Command::new(env!("CARGO_BIN_EXE_r_ust_cli"))
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
    assert!(stdout.contains("[1] 2 NA\n"));
    assert!(stdout.contains("[1] \"AB\"\n"));
    assert!(stdout.contains("[1] \"xy\"\n"));
    assert!(stdout.contains("[1] \"bc\" NA\n"));
    assert!(stdout.contains("[1] \"ab\" \"cde\"\n"));
    assert!(stdout.contains("[1] TRUE FALSE NA\n"));
    assert!(stdout.contains("[1] TRUE FALSE FALSE\n"));
    assert!(stdout.contains("[1] 1 3\n"));
    assert!(stdout.contains("[1] \"abc\" \"bcd\"\n"));
    assert!(stdout.contains("[1] \"a\" \"b\" \"c\"\n"));
    assert!(stdout.contains("[1] \"aXc\" \"Xdb\" NA\n"));
    assert!(stdout.contains("[1] \"aXc\" \"XdX\" NA\n"));
    assert!(stdout.contains("[1] \"aXc\"\n"));
    assert!(stdout.contains("[1] \"abc\" \"def\"\n"));
    assert!(stdout.contains("[1] \"a\" \"b\" NA\n"));
    assert!(stdout.contains("[1] \"a  \"\n"));
    assert!(stdout.contains("[1] \"  a\"\n"));
}

#[test]
fn expression_mode_runs_additional_string_helpers() {
    let output = Command::new(env!("CARGO_BIN_EXE_r_ust_cli"))
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
    assert!(stdout.contains("[1] FALSE TRUE TRUE\n"));
    assert!(stdout.contains("[1] FALSE TRUE NA\n"));
    assert!(stdout.contains("[1] \"ABrACAdABrA\" NA\n"));
    assert!(stdout.contains("[1] \"abc\" \"xy\" NA\n"));
    assert!(stdout.contains("[1] \"aaa\" \"bcbc\" NA\n"));
}

#[test]
fn expression_mode_runs_vector_order_helpers() {
    let output = Command::new(env!("CARGO_BIN_EXE_r_ust_cli"))
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
    assert!(stdout.contains("[1] 1 2 NaN\n"));
    assert!(stdout.contains("[1] FALSE FALSE TRUE FALSE TRUE\n"));
    assert!(stdout.contains("[1] \"a\" \"b\" \"c\"\n"));
}

#[test]
fn expression_mode_runs_set_helpers() {
    let output = Command::new(env!("CARGO_BIN_EXE_r_ust_cli"))
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
    assert!(stdout.contains("[1] 1 2 NaN 3\n"));
    assert!(stdout.contains("[1] 2 NaN\n"));
    assert!(stdout.contains("[1] 1\n"));
    assert!(stdout.contains("[1] TRUE\n"));
    assert!(stdout.contains("[1] FALSE\n"));
    assert!(stdout.contains("[1] \"a\" \"b\" NA \"c\"\n"));
    assert!(stdout.contains("[1] \"b\" NA\n"));
    assert!(stdout.contains("[1] \"a\"\n"));
    assert!(stdout.contains("[1] 3\n"));
    assert!(stdout.contains("[1] 1\n"));
    assert!(stdout.contains("[1] 2\n"));
}

#[test]
fn expression_mode_runs_position_sortedness_and_tabulate_helpers() {
    let output = Command::new(env!("CARGO_BIN_EXE_r_ust_cli"))
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

#[test]
fn expression_mode_runs_matrix_helpers() {
    let output = Command::new(env!("CARGO_BIN_EXE_r_ust_cli"))
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
    let output = Command::new(env!("CARGO_BIN_EXE_r_ust_cli"))
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
    let output = Command::new(env!("CARGO_BIN_EXE_r_ust_cli"))
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

#[test]
fn expression_mode_runs_extract_and_dollar_assignment() {
    let output = Command::new(env!("CARGO_BIN_EXE_r_ust_cli"))
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
    assert!(stdout.contains("[1] \"a\" \"renamed\"\n"));
}

#[test]
fn version_mode_reports_runtime_target() {
    let output = Command::new(env!("CARGO_BIN_EXE_r_ust_cli"))
        .arg("--version")
        .output()
        .expect("run r_ust_cli");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains("R-ust 0.1.0"));
    assert!(stdout.contains("R 4.6.0"));
}
