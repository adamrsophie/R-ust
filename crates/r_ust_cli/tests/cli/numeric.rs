use super::batch_command;

#[test]
fn expression_mode_runs_arithmetic_and_rmath() {
    let output = batch_command()
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
fn expression_mode_runs_base_numeric_helpers() {
    let output = batch_command()
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
    assert!(stdout.contains("[1] FALSE  TRUE FALSE\n"));
    assert!(stdout.contains("[1]  TRUE FALSE FALSE FALSE\n"));
    assert!(stdout.contains("[1] FALSE FALSE  TRUE  TRUE\n"));
    assert!(stdout.contains("[1]  -2   1 NaN\n"));
    assert!(stdout.contains("[1]  -1   2 NaN\n"));
    assert!(stdout.contains("[1]  -1   1 NaN\n"));
    assert!(stdout.contains("[1]  -1   0   1 NaN\n"));
    assert!(stdout.contains("[1] 0 1 2\n"));
    assert!(stdout.contains("[1] 0 1 3\n"));
    assert!(stdout.contains("[1] 0 1\n"));
    assert!(stdout.contains("[1]  1 -1\n"));
    assert!(stdout.contains("[1] 1.2 5.7\n"));
    assert!(stdout.contains("[1] 1200\n"));
}

#[test]
fn expression_mode_runs_special_numeric_helpers() {
    let output = batch_command()
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
    assert!(stdout.contains("[1] 1.570796326795  3.14159265359\n"));
    assert!(stdout.contains("[1]              0 1.175201193644\n"));
    assert!(stdout.contains("[1] 1\n"));
    assert!(stdout.contains("[1]              0 0.761594155956\n"));
    assert!(stdout.contains("[1] 0.88137358702\n"));
    assert!(stdout.contains("[1] 0.549306144334\n"));
    assert!(stdout.contains("[1]  1.23 12300\n"));
    assert!(stdout.contains("[1]   1 120\n"));
    assert!(stdout.contains("[1] 4.787491742782\n"));
    assert!(stdout.contains("[1] -2.484906649788\n"));
    assert!(stdout.contains("[1] 2.302585092994\n"));
    assert!(stdout.contains("[1] -0.577215664902\n"));
    assert!(stdout.contains("[1] 1.644934066848"));
}

#[test]
fn expression_mode_runs_uniform_sampling_helpers() {
    let output = batch_command()
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
    let output = batch_command()
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
    assert!(stdout.contains("[1]  6.835960082702 -4.780133104908\n"));
    assert!(stdout.contains("[1] 0.688256497741 5.147557532988\n"));
    assert!(stdout.contains("[1] 6.149202253105 3.471663233511\n"));
    assert!(stdout.contains("[1] 3 2\n"));
    assert!(stdout.contains("[1] 0 7\n"));
    assert!(stdout.contains("[1] 5 5\n"));
    assert!(stdout.contains("[1] 3.159988069084  1.17965232797\n"));
    assert!(stdout.contains("[1]  9.57552211355 2.852550537173\n"));
    assert!(stdout.contains("[1] 0.669891847819 0.279765194257\n"));
    assert!(stdout.contains("[1] 3.043965334059 0.582654140066\n"));
    assert!(stdout.contains("[1] 1.498005549196 0.067031526736\n"));
    assert!(stdout.contains("[1] 9 4\n"));
    assert!(stdout.contains("[1] 3 4\n"));
    assert!(stdout.contains("[1]   7.0961758966 0.521138258407\n"));
    assert_eq!(stdout.matches("[1] 2 3 1\n").count(), 2);
    assert!(stdout.contains("[1] 6 8 2\n"));
    assert!(stdout.contains("[1] 8 3 3\n"));
}
