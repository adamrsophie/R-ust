use r_ust_nmath as rmath;

fn assert_close(actual: f64, expected: f64, tol: f64) {
    if actual.is_nan() || expected.is_nan() {
        assert!(
            actual.is_nan() && expected.is_nan(),
            "nan mismatch: actual={actual:?} expected={expected:?}"
        );
        return;
    }
    if actual.is_infinite() || expected.is_infinite() {
        assert_eq!(actual, expected);
        return;
    }
    let scale = expected.abs().max(1.0);
    assert!(
        (actual - expected).abs() <= tol * scale,
        "actual={actual:?} expected={expected:?} diff={:?}",
        actual - expected
    );
}

#[test]
fn normal_reference_values_are_stable() {
    let q70 = rmath::qnorm(0.7, 0.0, 1.0, true, false);

    assert_close(
        rmath::dnorm(0.7, 0.0, 1.0, false),
        0.31225393336676127,
        2e-15,
    );
    assert_close(
        rmath::dnorm(0.7, 0.0, 1.0, true),
        -1.1639385332046727,
        2e-15,
    );
    assert_close(q70, 0.5244005127080407, 3e-15);
    assert_close(rmath::pnorm(q70, 0.0, 1.0, true, false), 0.7, 2e-15);
}

#[test]
fn normal_boundaries_match_rmath_contract() {
    assert_eq!(rmath::dnorm(f64::INFINITY, 0.0, 1.0, false), 0.0);
    assert_eq!(rmath::dnorm(0.0, 0.0, 0.0, false), f64::INFINITY);
    assert_eq!(rmath::dnorm(1.0, 0.0, 0.0, false), 0.0);

    assert_eq!(rmath::pnorm(f64::NEG_INFINITY, 0.0, 1.0, true, false), 0.0);
    assert_eq!(rmath::pnorm(f64::INFINITY, 0.0, 1.0, true, false), 1.0);
    assert_eq!(rmath::qnorm(0.0, 0.0, 1.0, true, false), f64::NEG_INFINITY);
    assert_eq!(rmath::qnorm(1.0, 0.0, 1.0, true, false), f64::INFINITY);
}

#[test]
fn pow_di_handles_integer_boundaries_without_debug_overflow() {
    assert_close(rmath::R_pow_di(2.0, -3), 0.125, 0.0);
    assert_close(rmath::R_pow_di(-2.0, 3), -8.0, 0.0);
    assert_eq!(rmath::R_pow_di(2.0, i32::MIN), 0.0);
    assert_eq!(rmath::R_pow_di(-1.0, i32::MIN), 1.0);
}

#[test]
fn elementary_helpers_match_rmath_contracts() {
    assert_eq!(rmath::R_finite(1.0), 1);
    assert_eq!(rmath::R_finite(f64::INFINITY), 0);
    assert_eq!(rmath::R_isnancpp(f64::NAN), 1);
    assert_eq!(rmath::imax2(-4, 7), 7);
    assert_eq!(rmath::imin2(-4, 7), -4);
    assert_eq!(rmath::sign(-3.5), -1.0);
    assert_eq!(rmath::fsign(-3.5, 2.0), 3.5);
    assert_eq!(rmath::ftrunc(-3.9), -3.0);
    assert_eq!(rmath::cospi(0.5), 0.0);
    assert_eq!(rmath::sinpi(0.5), 1.0);
    assert_eq!(rmath::tanpi(0.25), 1.0);
    assert_close(rmath::log1pexp(40.0), 40.0, 1e-15);
    assert_close(rmath::log1pmx(1e-8), -4.999999966666667e-17, 1e-12);
    assert_close(
        rmath::logspace_add(2.0, 3.0),
        (2.0f64.exp() + 3.0f64.exp()).ln(),
        1e-15,
    );
    assert_close(rmath::pow1p(1e-12, 3.0), 1.000000000003, 1e-15);
}

#[test]
fn closed_form_distribution_round_trips_are_stable() {
    assert_close(rmath::dunif(0.25, 0.0, 2.0, false), 0.5, 0.0);
    assert_close(rmath::punif(0.25, 0.0, 2.0, true, false), 0.125, 0.0);
    assert_close(rmath::qunif(0.125, 0.0, 2.0, true, false), 0.25, 0.0);

    assert_close(
        rmath::qexp(rmath::pexp(2.0, 3.0, true, false), 3.0, true, false),
        2.0,
        2e-15,
    );
    assert_close(
        rmath::qcauchy(0.7, 0.0, 1.0, true, false),
        0.7265425280053609,
        3e-15,
    );
    assert_close(
        rmath::pcauchy(
            rmath::qcauchy(0.7, 0.0, 1.0, true, false),
            0.0,
            1.0,
            true,
            false,
        ),
        0.7,
        3e-15,
    );
    assert_close(
        rmath::plogis(
            rmath::qlogis(0.7, 0.0, 1.0, true, false),
            0.0,
            1.0,
            true,
            false,
        ),
        0.7,
        2e-15,
    );
    assert_close(
        rmath::pweibull(
            rmath::qweibull(0.7, 1.5, 3.0, true, false),
            1.5,
            3.0,
            true,
            false,
        ),
        0.7,
        2e-15,
    );
    assert_close(
        rmath::plnorm(
            rmath::qlnorm(0.7, 0.1, 0.7, true, false),
            0.1,
            0.7,
            true,
            false,
        ),
        0.7,
        3e-15,
    );
}

#[test]
fn gamma_choose_and_discrete_families_are_stable() {
    assert_close(rmath::gammafn(5.0), 24.0, 2e-15);
    assert_close(rmath::lgammafn(5.0), 24.0f64.ln(), 2e-15);
    let (lgamma, sign) = rmath::lgammafn_sign(-0.5);
    assert_eq!(sign, -1);
    assert_close(
        lgamma,
        std::f64::consts::PI.sqrt().ln() + 2.0f64.ln(),
        2e-15,
    );
    assert_close(rmath::beta(2.0, 3.0), 1.0 / 12.0, 2e-14);
    assert_close(rmath::choose(10.0, 3.0), 120.0, 0.0);
    assert_close(rmath::lchoose(10.0, 3.0), 120.0f64.ln(), 2e-14);

    assert_close(rmath::dbinom(3.0, 10.0, 0.2, false), 0.201326592, 2e-14);
    assert_close(
        rmath::pbinom(3.0, 10.0, 0.2, true, false),
        0.8791261184,
        2e-14,
    );
    assert_eq!(rmath::qbinom(0.5, 10.0, 0.2, true, false), 2.0);

    assert_close(rmath::dpois(3.0, 2.0, false), 0.1804470443154836, 2e-14);
    assert_close(
        rmath::ppois(3.0, 2.0, true, false),
        0.857123460498547,
        2e-14,
    );
    assert_eq!(rmath::qpois(0.8, 2.0, true, false), 3.0);

    assert_close(rmath::dgeom(3.0, 0.25, false), 0.10546875, 0.0);
    assert_close(rmath::pgeom(3.0, 0.25, true, false), 0.68359375, 2e-15);
    assert_eq!(rmath::qgeom(0.7, 0.25, true, false), 4.0);
}

#[test]
fn gamma_based_density_families_are_stable() {
    assert_close(rmath::dbeta(0.3, 2.0, 3.0, false), 1.764, 2e-14);
    assert_close(
        rmath::dgamma(2.0, 3.0, 2.0, false),
        0.09196986029286064,
        2e-14,
    );
    assert_close(rmath::dchisq(2.0, 4.0, false), 0.18393972058572114, 2e-14);
    assert_close(rmath::dt(1.0, 10.0, false), 0.230361989229139, 2e-12);
    assert_close(rmath::df(1.0, 5.0, 10.0, false), 0.4954797834866395, 2e-12);
    assert_close(rmath::dnbinom(3.0, 5.0, 0.4, false), 0.0774144, 2e-14);
    assert_close(
        rmath::dnbinom_mu(3.0, 5.0, 7.0, false),
        0.08724935351666703,
        2e-14,
    );
    assert_close(
        rmath::dnchisq(2.0, 4.0, 0.0, false),
        rmath::dchisq(2.0, 4.0, false),
        0.0,
    );
}

#[test]
fn cdf_and_quantile_layer_round_trips_are_stable() {
    let exp_case = 1.0 - (-2.0f64 / 3.0).exp();
    assert_close(rmath::pgamma(2.0, 1.0, 3.0, true, false), exp_case, 2e-14);
    assert_close(rmath::qgamma(exp_case, 1.0, 3.0, true, false), 2.0, 2e-12);
    assert_close(
        rmath::pchisq(2.0, 2.0, true, false),
        1.0 - (-1.0f64).exp(),
        2e-14,
    );
    assert_close(
        rmath::qchisq(1.0 - (-1.0f64).exp(), 2.0, true, false),
        2.0,
        2e-12,
    );

    assert_close(rmath::pbeta(0.3, 2.0, 3.0, true, false), 0.3483, 2e-13);
    assert_close(rmath::qbeta(0.3483, 2.0, 3.0, true, false), 0.3, 2e-12);

    assert_close(rmath::pf(1.0, 2.0, 2.0, true, false), 0.5, 2e-14);
    assert_close(rmath::qf(0.5, 2.0, 2.0, true, false), 1.0, 2e-12);
    assert_close(rmath::pt(1.0, 1.0, true, false), 0.75, 2e-14);
    assert_close(rmath::qt(0.75, 1.0, true, false), 1.0, 2e-12);

    let pnb = rmath::pnbinom(3.0, 5.0, 0.4, true, false);
    assert_close(
        rmath::pnbinom(
            rmath::qnbinom(pnb, 5.0, 0.4, true, false),
            5.0,
            0.4,
            true,
            false,
        ),
        pnb,
        5e-2,
    );
    let pnb_mu = rmath::pnbinom_mu(3.0, 5.0, 7.0, true, false);
    assert_close(
        rmath::pnbinom_mu(
            rmath::qnbinom_mu(pnb_mu, 5.0, 7.0, true, false),
            5.0,
            7.0,
            true,
            false,
        ),
        pnb_mu,
        5e-2,
    );
}

#[test]
fn remaining_rmath_surface_smoke_cases_are_stable() {
    assert_eq!(rmath::fround(2.5, 0.0), 2.0);
    assert_eq!(rmath::fround(3.5, 0.0), 4.0);
    assert_eq!(rmath::fprec(1234.0, 2.0), 1200.0);

    assert_close(rmath::digamma(1.0), -0.5772156649015329, 2e-14);
    assert_close(
        rmath::trigamma(1.0),
        std::f64::consts::PI.powi(2) / 6.0,
        2e-12,
    );
    assert_close(rmath::tetragamma(1.0), -2.4041138063191885, 2e-12);
    assert_close(
        rmath::pentagamma(1.0),
        std::f64::consts::PI.powi(4) / 15.0,
        2e-12,
    );
    assert_close(rmath::psigamma(1.0, 0.0), rmath::digamma(1.0), 0.0);
    let (psi, nz, ierr) = rmath::dpsifn(1.0, 0, 1, 1);
    assert_eq!((nz, ierr), (0, 0));
    assert_close(psi[0], -rmath::digamma(1.0), 2e-14);

    assert_close(
        rmath::dhyper(2.0, 5.0, 7.0, 4.0, false),
        420.0 / 990.0,
        2e-14,
    );
    assert_close(
        rmath::phyper(2.0, 5.0, 7.0, 4.0, true, false),
        420.0 / 495.0,
        2e-14,
    );
    assert_eq!(rmath::qhyper(0.5, 5.0, 7.0, 4.0, true, false), 2.0);

    assert_close(rmath::dsignrank(3.0, 3.0, false), 0.25, 1e-15);
    assert_close(rmath::psignrank(3.0, 3.0, true, false), 0.625, 1e-15);
    assert_eq!(rmath::qsignrank(0.625, 3.0, true, false), 3.0);
    rmath::signrank_free();

    assert_close(rmath::dwilcox(3.0, 2.0, 3.0, false), 0.2, 1e-15);
    assert_close(rmath::pwilcox(3.0, 2.0, 3.0, true, false), 0.6, 1e-15);
    assert_eq!(rmath::qwilcox(0.6, 2.0, 3.0, true, false), 3.0);
    rmath::wilcox_free();

    assert_close(rmath::bessel_j(0.0, 0.0), 1.0, 0.0);
    assert_close(rmath::bessel_i(0.0, 0.0, 0.0), 1.0, 0.0);
    assert_close(rmath::bessel_j(1.0, 0.0), 0.7651976865579666, 2e-14);
    assert_close(rmath::bessel_i(1.0, 0.0, 0.0), 1.2660658777520082, 2e-14);
    assert!(rmath::bessel_k(2.0, 0.5, 0.0).is_finite());
    assert!(rmath::bessel_y(2.0, 0.5).is_finite());

    assert_close(
        rmath::pnchisq(2.0, 4.0, 0.0, true, false),
        rmath::pchisq(2.0, 4.0, true, false),
        0.0,
    );
    assert_close(
        rmath::qnchisq(0.5, 4.0, 0.0, true, false),
        rmath::qchisq(0.5, 4.0, true, false),
        2e-12,
    );
    assert_close(
        rmath::pnbeta(0.3, 2.0, 3.0, 0.0, true, false),
        rmath::pbeta(0.3, 2.0, 3.0, true, false),
        0.0,
    );
    assert_close(
        rmath::dnbeta(0.3, 2.0, 3.0, 0.0, false),
        rmath::dbeta(0.3, 2.0, 3.0, false),
        0.0,
    );
    assert_close(
        rmath::pnf(1.0, 5.0, 10.0, 0.0, true, false),
        rmath::pf(1.0, 5.0, 10.0, true, false),
        0.0,
    );
    assert_close(
        rmath::dnf(1.0, 5.0, 10.0, 0.0, false),
        rmath::df(1.0, 5.0, 10.0, false),
        0.0,
    );
    assert_close(
        rmath::pnt(1.0, 10.0, 0.0, true, false),
        rmath::pt(1.0, 10.0, true, false),
        0.0,
    );
    assert_close(
        rmath::dnt(1.0, 10.0, 0.0, false),
        rmath::dt(1.0, 10.0, false),
        0.0,
    );
    assert_close(
        rmath::qnt(0.75, 1.0, 0.0, true, false),
        rmath::qt(0.75, 1.0, true, false),
        2e-12,
    );
    assert_close(
        rmath::qtukey(
            rmath::ptukey(2.0, 1.0, 3.0, 10.0, true, false),
            1.0,
            3.0,
            10.0,
            true,
            false,
        ),
        2.0,
        2e-10,
    );
}

#[test]
fn explicit_rng_state_and_global_state_match() {
    let mut explicit = rmath::RngState::new(123, 456);
    rmath::set_seed(123, 456);

    for _ in 0..20 {
        assert_close(explicit.unif_rand(), rmath::unif_rand(), 0.0);
    }
    assert_eq!(explicit.seed(), rmath::get_seed());

    let mut explicit = rmath::RngState::new(123, 456);
    rmath::set_seed(123, 456);
    assert_close(explicit.exp_rand(), rmath::exp_rand(), 0.0);
    assert_close(explicit.rexp(2.0), rmath::rexp(2.0), 0.0);
    assert_close(explicit.rcauchy(0.0, 1.0), rmath::rcauchy(0.0, 1.0), 0.0);
    assert_close(explicit.rlogis(0.0, 1.0), rmath::rlogis(0.0, 1.0), 0.0);
    assert_close(explicit.rweibull(1.5, 2.0), rmath::rweibull(1.5, 2.0), 0.0);
    assert_close(explicit.rlnorm(0.1, 0.7), rmath::rlnorm(0.1, 0.7), 0.0);
    assert_close(explicit.rpois(2.0), rmath::rpois(2.0), 0.0);
    assert_close(explicit.rgeom(0.25), rmath::rgeom(0.25), 0.0);
    assert_close(
        explicit.rhyper(5.0, 7.0, 4.0),
        rmath::rhyper(5.0, 7.0, 4.0),
        0.0,
    );
    assert_close(explicit.rbinom(10.0, 0.2), rmath::rbinom(10.0, 0.2), 0.0);
    assert_close(explicit.rgamma(2.0, 3.0), rmath::rgamma(2.0, 3.0), 0.0);
    assert_close(explicit.rchisq(4.0), rmath::rchisq(4.0), 0.0);
    assert_close(explicit.rbeta(2.0, 3.0), rmath::rbeta(2.0, 3.0), 0.0);
    assert_close(explicit.rf(5.0, 10.0), rmath::rf(5.0, 10.0), 0.0);
    assert_close(explicit.rt(10.0), rmath::rt(10.0), 0.0);
    assert_close(explicit.rnbinom(5.0, 0.4), rmath::rnbinom(5.0, 0.4), 0.0);
    assert_close(
        explicit.rnbinom_mu(5.0, 7.0),
        rmath::rnbinom_mu(5.0, 7.0),
        0.0,
    );
    assert_close(explicit.rnchisq(4.0, 2.0), rmath::rnchisq(4.0, 2.0), 0.0);
    assert_close(explicit.rsignrank(5.0), rmath::rsignrank(5.0), 0.0);
    assert_close(explicit.rwilcox(3.0, 4.0), rmath::rwilcox(3.0, 4.0), 0.0);

    let mut explicit_out = [0; 3];
    let mut global_out = [0; 3];
    explicit.rmultinom(12, &[0.2, 0.3, 0.5], &mut explicit_out);
    rmath::rmultinom(12, &[0.2, 0.3, 0.5], &mut global_out);
    assert_eq!(explicit_out, global_out);
}

#[test]
fn rmath_surface_tracks_unimplemented_backlog() {
    let implemented: std::collections::BTreeSet<_> =
        rmath::IMPLEMENTED_FUNCTIONS.iter().copied().collect();
    let unimplemented: std::collections::BTreeSet<_> =
        rmath::UNIMPLEMENTED_FUNCTIONS.iter().copied().collect();
    let public: std::collections::BTreeSet<_> = rmath::PUBLIC_FUNCTIONS.iter().copied().collect();

    assert!(implemented.is_disjoint(&unimplemented));
    assert_eq!(
        implemented
            .union(&unimplemented)
            .copied()
            .collect::<std::collections::BTreeSet<_>>(),
        public
    );
}
