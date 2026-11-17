use super::batch_command;

#[test]
fn expression_mode_runs_additional_distribution_families() {
    let output = batch_command()
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
    assert!(stdout.contains("[1] 0.156874\n"));
    assert!(stdout.contains("[1] 0.7558914\n"));
    assert!(stdout.contains("[1] 1\n"));
    assert!(stdout.contains("[1] 0.1654682\n"));
    assert!(stdout.contains("[1] 0.1175031\n"));
    assert!(stdout.contains("[1] 3.539988\n"));
    assert!(stdout.contains("[1] -0.8630462 -1.1507283\n"));
}

#[test]
fn expression_mode_runs_continuous_distribution_families() {
    let output = batch_command()
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
    assert!(stdout.contains("[1] 0.2617188\n"));
    assert!(stdout.contains("[1] 0.3857276\n"));
    assert!(stdout.contains("[1] 0.1839397\n"));
    assert!(stdout.contains("[1] 0.2642411\n"));
    assert!(stdout.contains("[1] 3.356694\n"));
    assert!(stdout.contains("[1] 0.2196798\n"));
    assert!(stdout.contains("[1] 0.8183913\n"));
    assert!(stdout.contains("[1] 0.7266868\n"));
    assert!(stdout.contains("[1] 0.1542215\n"));
    assert!(stdout.contains("[1] 0.786339\n"));
    assert!(stdout.contains("[1] 0.9419133\n"));
}

#[test]
fn expression_mode_runs_gamma_and_hypergeometric_distribution_families() {
    let output = batch_command()
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
    assert!(stdout.contains("[1] 0.01438768\n"));
    assert!(stdout.contains("[1] 10.69624\n"));
    assert!(stdout.contains("[1] 0.4242424\n"));
    assert!(stdout.contains("[1] 0.8484848\n"));
    assert!(stdout.contains("[1] 2\n"));
}

#[test]
fn expression_mode_runs_negative_binomial_and_rank_distribution_families() {
    let output = batch_command()
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
    assert!(stdout.contains("[1] 0.162614\n"));
    assert!(stdout.contains("[1] 0.4899473\n"));
    assert!(stdout.contains("[1] 4\n"));
    assert!(stdout.contains("[1] 0.125\n"));
    assert!(stdout.contains("[1] 0.3125\n"));
    assert!(stdout.contains("[1] 5\n"));
    assert!(stdout.contains("[1] 0.08571429\n"));
    assert!(stdout.contains("[1] 0.2\n"));
    assert!(stdout.contains("[1] 6\n"));
}

#[test]
fn expression_mode_runs_noncentral_distribution_forms() {
    let output = batch_command()
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
    assert!(stdout.contains("[1] 1.386419\n"));
    assert!(stdout.contains("[1] 0.1931533\n"));
    assert!(stdout.contains("[1] 0.4416064\n"));
    assert!(stdout.contains("[1] 0.1418799\n"));
    assert!(stdout.contains("[1] 0.1861098\n"));
    assert!(stdout.contains("[1] 4.234893\n"));
    assert!(stdout.contains("[1] 0.3483969\n"));
    assert!(stdout.contains("[1] 0.5758097\n"));
    assert!(stdout.contains("[1] 1.579344\n"));
    assert!(stdout.contains("[1] 0.1830228\n"));
    assert!(stdout.contains("[1] 0.7104069\n"));
    assert!(stdout.contains("[1] 1.186277\n"));
}
