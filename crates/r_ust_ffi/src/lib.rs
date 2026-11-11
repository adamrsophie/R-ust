use std::os::raw::{c_double, c_int, c_uint};

#[no_mangle]
pub extern "C" fn dnorm(x: c_double, mean: c_double, sd: c_double, give_log: c_int) -> c_double {
    r_ust_nmath::dnorm(x, mean, sd, give_log != 0)
}

#[no_mangle]
pub extern "C" fn dunif(x: c_double, a: c_double, b: c_double, give_log: c_int) -> c_double {
    r_ust_nmath::dunif(x, a, b, give_log != 0)
}

#[no_mangle]
pub extern "C" fn dexp(x: c_double, scale: c_double, give_log: c_int) -> c_double {
    r_ust_nmath::dexp(x, scale, give_log != 0)
}

#[no_mangle]
pub extern "C" fn dcauchy(
    x: c_double,
    location: c_double,
    scale: c_double,
    give_log: c_int,
) -> c_double {
    r_ust_nmath::dcauchy(x, location, scale, give_log != 0)
}

#[no_mangle]
pub extern "C" fn dlogis(
    x: c_double,
    location: c_double,
    scale: c_double,
    give_log: c_int,
) -> c_double {
    r_ust_nmath::dlogis(x, location, scale, give_log != 0)
}

#[no_mangle]
pub extern "C" fn dweibull(
    x: c_double,
    shape: c_double,
    scale: c_double,
    give_log: c_int,
) -> c_double {
    r_ust_nmath::dweibull(x, shape, scale, give_log != 0)
}

#[no_mangle]
pub extern "C" fn dlnorm(
    x: c_double,
    meanlog: c_double,
    sdlog: c_double,
    give_log: c_int,
) -> c_double {
    r_ust_nmath::dlnorm(x, meanlog, sdlog, give_log != 0)
}

#[no_mangle]
pub extern "C" fn dbinom(x: c_double, n: c_double, p: c_double, give_log: c_int) -> c_double {
    r_ust_nmath::dbinom(x, n, p, give_log != 0)
}

#[no_mangle]
pub extern "C" fn dbinom_raw(
    x: c_double,
    n: c_double,
    p: c_double,
    q: c_double,
    give_log: c_int,
) -> c_double {
    r_ust_nmath::dbinom_raw(x, n, p, q, give_log != 0)
}

#[no_mangle]
pub extern "C" fn dpois(x: c_double, lambda: c_double, give_log: c_int) -> c_double {
    r_ust_nmath::dpois(x, lambda, give_log != 0)
}

#[no_mangle]
pub extern "C" fn dpois_raw(x: c_double, lambda: c_double, give_log: c_int) -> c_double {
    r_ust_nmath::dpois_raw(x, lambda, give_log != 0)
}

#[no_mangle]
pub extern "C" fn dgeom(x: c_double, p: c_double, give_log: c_int) -> c_double {
    r_ust_nmath::dgeom(x, p, give_log != 0)
}

#[no_mangle]
pub extern "C" fn dbeta(x: c_double, a: c_double, b: c_double, give_log: c_int) -> c_double {
    r_ust_nmath::dbeta(x, a, b, give_log != 0)
}

#[no_mangle]
pub extern "C" fn dgamma(
    x: c_double,
    shape: c_double,
    scale: c_double,
    give_log: c_int,
) -> c_double {
    r_ust_nmath::dgamma(x, shape, scale, give_log != 0)
}

#[no_mangle]
pub extern "C" fn dchisq(x: c_double, df: c_double, give_log: c_int) -> c_double {
    r_ust_nmath::dchisq(x, df, give_log != 0)
}

#[no_mangle]
pub extern "C" fn dt(x: c_double, n: c_double, give_log: c_int) -> c_double {
    r_ust_nmath::dt(x, n, give_log != 0)
}

#[no_mangle]
pub extern "C" fn df(x: c_double, m: c_double, n: c_double, give_log: c_int) -> c_double {
    r_ust_nmath::df(x, m, n, give_log != 0)
}

#[no_mangle]
pub extern "C" fn dnbinom(
    x: c_double,
    size: c_double,
    prob: c_double,
    give_log: c_int,
) -> c_double {
    r_ust_nmath::dnbinom(x, size, prob, give_log != 0)
}

#[no_mangle]
pub extern "C" fn dnbinom_mu(
    x: c_double,
    size: c_double,
    mu: c_double,
    give_log: c_int,
) -> c_double {
    r_ust_nmath::dnbinom_mu(x, size, mu, give_log != 0)
}

#[no_mangle]
pub extern "C" fn dnchisq(x: c_double, df: c_double, ncp: c_double, give_log: c_int) -> c_double {
    r_ust_nmath::dnchisq(x, df, ncp, give_log != 0)
}

#[no_mangle]
pub extern "C" fn pnorm(
    q: c_double,
    mean: c_double,
    sd: c_double,
    lower_tail: c_int,
    log_p: c_int,
) -> c_double {
    r_ust_nmath::pnorm(q, mean, sd, lower_tail != 0, log_p != 0)
}

#[no_mangle]
pub unsafe extern "C" fn pnorm_both(
    x: c_double,
    cum: *mut c_double,
    ccum: *mut c_double,
    i_tail: c_int,
    log_p: c_int,
) {
    let (left, right) = r_ust_nmath::pnorm_both(x, i_tail, log_p != 0);
    if !cum.is_null() {
        *cum = left;
    }
    if !ccum.is_null() {
        *ccum = right;
    }
}

#[no_mangle]
pub extern "C" fn punif(
    x: c_double,
    a: c_double,
    b: c_double,
    lower_tail: c_int,
    log_p: c_int,
) -> c_double {
    r_ust_nmath::punif(x, a, b, lower_tail != 0, log_p != 0)
}

#[no_mangle]
pub extern "C" fn pexp(x: c_double, scale: c_double, lower_tail: c_int, log_p: c_int) -> c_double {
    r_ust_nmath::pexp(x, scale, lower_tail != 0, log_p != 0)
}

#[no_mangle]
pub extern "C" fn pcauchy(
    x: c_double,
    location: c_double,
    scale: c_double,
    lower_tail: c_int,
    log_p: c_int,
) -> c_double {
    r_ust_nmath::pcauchy(x, location, scale, lower_tail != 0, log_p != 0)
}

#[no_mangle]
pub extern "C" fn plogis(
    x: c_double,
    location: c_double,
    scale: c_double,
    lower_tail: c_int,
    log_p: c_int,
) -> c_double {
    r_ust_nmath::plogis(x, location, scale, lower_tail != 0, log_p != 0)
}

#[no_mangle]
pub extern "C" fn pweibull(
    x: c_double,
    shape: c_double,
    scale: c_double,
    lower_tail: c_int,
    log_p: c_int,
) -> c_double {
    r_ust_nmath::pweibull(x, shape, scale, lower_tail != 0, log_p != 0)
}

#[no_mangle]
pub extern "C" fn plnorm(
    x: c_double,
    meanlog: c_double,
    sdlog: c_double,
    lower_tail: c_int,
    log_p: c_int,
) -> c_double {
    r_ust_nmath::plnorm(x, meanlog, sdlog, lower_tail != 0, log_p != 0)
}

#[no_mangle]
pub extern "C" fn pbinom(
    x: c_double,
    n: c_double,
    p: c_double,
    lower_tail: c_int,
    log_p: c_int,
) -> c_double {
    r_ust_nmath::pbinom(x, n, p, lower_tail != 0, log_p != 0)
}

#[no_mangle]
pub extern "C" fn ppois(
    x: c_double,
    lambda: c_double,
    lower_tail: c_int,
    log_p: c_int,
) -> c_double {
    r_ust_nmath::ppois(x, lambda, lower_tail != 0, log_p != 0)
}

#[no_mangle]
pub extern "C" fn pgeom(x: c_double, p: c_double, lower_tail: c_int, log_p: c_int) -> c_double {
    r_ust_nmath::pgeom(x, p, lower_tail != 0, log_p != 0)
}

#[no_mangle]
pub extern "C" fn pgamma(
    x: c_double,
    shape: c_double,
    scale: c_double,
    lower_tail: c_int,
    log_p: c_int,
) -> c_double {
    r_ust_nmath::pgamma(x, shape, scale, lower_tail != 0, log_p != 0)
}

#[no_mangle]
pub extern "C" fn pchisq(x: c_double, df: c_double, lower_tail: c_int, log_p: c_int) -> c_double {
    r_ust_nmath::pchisq(x, df, lower_tail != 0, log_p != 0)
}

#[no_mangle]
pub extern "C" fn pbeta(
    x: c_double,
    a: c_double,
    b: c_double,
    lower_tail: c_int,
    log_p: c_int,
) -> c_double {
    r_ust_nmath::pbeta(x, a, b, lower_tail != 0, log_p != 0)
}

#[no_mangle]
pub extern "C" fn pf(
    x: c_double,
    m: c_double,
    n: c_double,
    lower_tail: c_int,
    log_p: c_int,
) -> c_double {
    r_ust_nmath::pf(x, m, n, lower_tail != 0, log_p != 0)
}

#[no_mangle]
pub extern "C" fn pt(x: c_double, df: c_double, lower_tail: c_int, log_p: c_int) -> c_double {
    r_ust_nmath::pt(x, df, lower_tail != 0, log_p != 0)
}

#[no_mangle]
pub extern "C" fn pnbinom(
    x: c_double,
    size: c_double,
    prob: c_double,
    lower_tail: c_int,
    log_p: c_int,
) -> c_double {
    r_ust_nmath::pnbinom(x, size, prob, lower_tail != 0, log_p != 0)
}

#[no_mangle]
pub extern "C" fn pnbinom_mu(
    x: c_double,
    size: c_double,
    mu: c_double,
    lower_tail: c_int,
    log_p: c_int,
) -> c_double {
    r_ust_nmath::pnbinom_mu(x, size, mu, lower_tail != 0, log_p != 0)
}

#[no_mangle]
pub extern "C" fn qnorm(
    p: c_double,
    mean: c_double,
    sd: c_double,
    lower_tail: c_int,
    log_p: c_int,
) -> c_double {
    r_ust_nmath::qnorm(p, mean, sd, lower_tail != 0, log_p != 0)
}

#[no_mangle]
pub extern "C" fn qunif(
    p: c_double,
    a: c_double,
    b: c_double,
    lower_tail: c_int,
    log_p: c_int,
) -> c_double {
    r_ust_nmath::qunif(p, a, b, lower_tail != 0, log_p != 0)
}

#[no_mangle]
pub extern "C" fn qexp(p: c_double, scale: c_double, lower_tail: c_int, log_p: c_int) -> c_double {
    r_ust_nmath::qexp(p, scale, lower_tail != 0, log_p != 0)
}

#[no_mangle]
pub extern "C" fn qcauchy(
    p: c_double,
    location: c_double,
    scale: c_double,
    lower_tail: c_int,
    log_p: c_int,
) -> c_double {
    r_ust_nmath::qcauchy(p, location, scale, lower_tail != 0, log_p != 0)
}

#[no_mangle]
pub extern "C" fn qlogis(
    p: c_double,
    location: c_double,
    scale: c_double,
    lower_tail: c_int,
    log_p: c_int,
) -> c_double {
    r_ust_nmath::qlogis(p, location, scale, lower_tail != 0, log_p != 0)
}

#[no_mangle]
pub extern "C" fn qweibull(
    p: c_double,
    shape: c_double,
    scale: c_double,
    lower_tail: c_int,
    log_p: c_int,
) -> c_double {
    r_ust_nmath::qweibull(p, shape, scale, lower_tail != 0, log_p != 0)
}

#[no_mangle]
pub extern "C" fn qlnorm(
    p: c_double,
    meanlog: c_double,
    sdlog: c_double,
    lower_tail: c_int,
    log_p: c_int,
) -> c_double {
    r_ust_nmath::qlnorm(p, meanlog, sdlog, lower_tail != 0, log_p != 0)
}

#[no_mangle]
pub extern "C" fn qbinom(
    p: c_double,
    n: c_double,
    prob: c_double,
    lower_tail: c_int,
    log_p: c_int,
) -> c_double {
    r_ust_nmath::qbinom(p, n, prob, lower_tail != 0, log_p != 0)
}

#[no_mangle]
pub extern "C" fn qpois(
    p: c_double,
    lambda: c_double,
    lower_tail: c_int,
    log_p: c_int,
) -> c_double {
    r_ust_nmath::qpois(p, lambda, lower_tail != 0, log_p != 0)
}

#[no_mangle]
pub extern "C" fn qgeom(p: c_double, prob: c_double, lower_tail: c_int, log_p: c_int) -> c_double {
    r_ust_nmath::qgeom(p, prob, lower_tail != 0, log_p != 0)
}

#[no_mangle]
pub extern "C" fn qgamma(
    p: c_double,
    shape: c_double,
    scale: c_double,
    lower_tail: c_int,
    log_p: c_int,
) -> c_double {
    r_ust_nmath::qgamma(p, shape, scale, lower_tail != 0, log_p != 0)
}

#[no_mangle]
pub extern "C" fn qchisq(p: c_double, df: c_double, lower_tail: c_int, log_p: c_int) -> c_double {
    r_ust_nmath::qchisq(p, df, lower_tail != 0, log_p != 0)
}

#[no_mangle]
pub extern "C" fn qbeta(
    p: c_double,
    a: c_double,
    b: c_double,
    lower_tail: c_int,
    log_p: c_int,
) -> c_double {
    r_ust_nmath::qbeta(p, a, b, lower_tail != 0, log_p != 0)
}

#[no_mangle]
pub extern "C" fn qf(
    p: c_double,
    m: c_double,
    n: c_double,
    lower_tail: c_int,
    log_p: c_int,
) -> c_double {
    r_ust_nmath::qf(p, m, n, lower_tail != 0, log_p != 0)
}

#[no_mangle]
pub extern "C" fn qt(p: c_double, df: c_double, lower_tail: c_int, log_p: c_int) -> c_double {
    r_ust_nmath::qt(p, df, lower_tail != 0, log_p != 0)
}

#[no_mangle]
pub extern "C" fn qnbinom(
    p: c_double,
    size: c_double,
    prob: c_double,
    lower_tail: c_int,
    log_p: c_int,
) -> c_double {
    r_ust_nmath::qnbinom(p, size, prob, lower_tail != 0, log_p != 0)
}

#[no_mangle]
pub extern "C" fn qnbinom_mu(
    p: c_double,
    size: c_double,
    mu: c_double,
    lower_tail: c_int,
    log_p: c_int,
) -> c_double {
    r_ust_nmath::qnbinom_mu(p, size, mu, lower_tail != 0, log_p != 0)
}

#[no_mangle]
pub extern "C" fn set_seed(i1: c_uint, i2: c_uint) {
    r_ust_nmath::set_seed(i1, i2);
}

#[no_mangle]
pub unsafe extern "C" fn get_seed(i1: *mut c_uint, i2: *mut c_uint) {
    let (seed1, seed2) = r_ust_nmath::get_seed();
    if !i1.is_null() {
        *i1 = seed1;
    }
    if !i2.is_null() {
        *i2 = seed2;
    }
}

#[no_mangle]
pub extern "C" fn unif_rand() -> c_double {
    r_ust_nmath::unif_rand()
}

#[no_mangle]
pub extern "C" fn norm_rand() -> c_double {
    r_ust_nmath::norm_rand()
}

#[no_mangle]
pub extern "C" fn exp_rand() -> c_double {
    r_ust_nmath::exp_rand()
}

#[no_mangle]
pub extern "C" fn runif(min: c_double, max: c_double) -> c_double {
    r_ust_nmath::runif(min, max)
}

#[no_mangle]
pub extern "C" fn rnorm(mean: c_double, sd: c_double) -> c_double {
    r_ust_nmath::rnorm(mean, sd)
}

#[no_mangle]
pub extern "C" fn rexp(scale: c_double) -> c_double {
    r_ust_nmath::rexp(scale)
}

#[no_mangle]
pub extern "C" fn rcauchy(location: c_double, scale: c_double) -> c_double {
    r_ust_nmath::rcauchy(location, scale)
}

#[no_mangle]
pub extern "C" fn rlogis(location: c_double, scale: c_double) -> c_double {
    r_ust_nmath::rlogis(location, scale)
}

#[no_mangle]
pub extern "C" fn rweibull(shape: c_double, scale: c_double) -> c_double {
    r_ust_nmath::rweibull(shape, scale)
}

#[no_mangle]
pub extern "C" fn rlnorm(meanlog: c_double, sdlog: c_double) -> c_double {
    r_ust_nmath::rlnorm(meanlog, sdlog)
}

#[no_mangle]
pub extern "C" fn rbinom(n: c_double, p: c_double) -> c_double {
    r_ust_nmath::rbinom(n, p)
}

#[no_mangle]
pub extern "C" fn rpois(lambda: c_double) -> c_double {
    r_ust_nmath::rpois(lambda)
}

#[no_mangle]
pub extern "C" fn rgeom(p: c_double) -> c_double {
    r_ust_nmath::rgeom(p)
}

#[no_mangle]
pub extern "C" fn rgamma(shape: c_double, scale: c_double) -> c_double {
    r_ust_nmath::rgamma(shape, scale)
}

#[no_mangle]
pub extern "C" fn rchisq(df: c_double) -> c_double {
    r_ust_nmath::rchisq(df)
}

#[no_mangle]
pub extern "C" fn rbeta(a: c_double, b: c_double) -> c_double {
    r_ust_nmath::rbeta(a, b)
}

#[no_mangle]
pub extern "C" fn rf(m: c_double, n: c_double) -> c_double {
    r_ust_nmath::rf(m, n)
}

#[no_mangle]
pub extern "C" fn rt(df: c_double) -> c_double {
    r_ust_nmath::rt(df)
}

#[no_mangle]
pub extern "C" fn rnbinom(size: c_double, prob: c_double) -> c_double {
    r_ust_nmath::rnbinom(size, prob)
}

#[no_mangle]
pub extern "C" fn rnbinom_mu(size: c_double, mu: c_double) -> c_double {
    r_ust_nmath::rnbinom_mu(size, mu)
}

#[no_mangle]
pub extern "C" fn rnchisq(df: c_double, ncp: c_double) -> c_double {
    r_ust_nmath::rnchisq(df, ncp)
}

#[no_mangle]
pub extern "C" fn R_pow(x: c_double, y: c_double) -> c_double {
    r_ust_nmath::R_pow(x, y)
}

#[no_mangle]
pub extern "C" fn R_pow_di(x: c_double, n: c_int) -> c_double {
    r_ust_nmath::R_pow_di(x, n)
}

#[no_mangle]
pub extern "C" fn Rlog1p(x: c_double) -> c_double {
    r_ust_nmath::Rlog1p(x)
}

#[no_mangle]
pub extern "C" fn R_unif_index(dn: c_double) -> c_double {
    r_ust_nmath::R_unif_index(dn)
}

#[no_mangle]
pub extern "C" fn R_finite(x: c_double) -> c_int {
    r_ust_nmath::R_finite(x)
}

#[no_mangle]
pub extern "C" fn R_isnancpp(x: c_double) -> c_int {
    r_ust_nmath::R_isnancpp(x)
}

#[no_mangle]
pub extern "C" fn Rtanpi(x: c_double) -> c_double {
    r_ust_nmath::Rtanpi(x)
}

#[no_mangle]
pub extern "C" fn cospi(x: c_double) -> c_double {
    r_ust_nmath::cospi(x)
}

#[no_mangle]
pub extern "C" fn sinpi(x: c_double) -> c_double {
    r_ust_nmath::sinpi(x)
}

#[no_mangle]
pub extern "C" fn tanpi(x: c_double) -> c_double {
    r_ust_nmath::tanpi(x)
}

#[no_mangle]
pub extern "C" fn fmax2(x: c_double, y: c_double) -> c_double {
    r_ust_nmath::fmax2(x, y)
}

#[no_mangle]
pub extern "C" fn fmin2(x: c_double, y: c_double) -> c_double {
    r_ust_nmath::fmin2(x, y)
}

#[no_mangle]
pub extern "C" fn imax2(x: c_int, y: c_int) -> c_int {
    r_ust_nmath::imax2(x, y)
}

#[no_mangle]
pub extern "C" fn imin2(x: c_int, y: c_int) -> c_int {
    r_ust_nmath::imin2(x, y)
}

#[no_mangle]
pub extern "C" fn sign(x: c_double) -> c_double {
    r_ust_nmath::sign(x)
}

#[no_mangle]
pub extern "C" fn fsign(x: c_double, y: c_double) -> c_double {
    r_ust_nmath::fsign(x, y)
}

#[no_mangle]
pub extern "C" fn ftrunc(x: c_double) -> c_double {
    r_ust_nmath::ftrunc(x)
}

#[no_mangle]
pub extern "C" fn log1pexp(x: c_double) -> c_double {
    r_ust_nmath::log1pexp(x)
}

#[no_mangle]
pub extern "C" fn log1mexp(x: c_double) -> c_double {
    r_ust_nmath::log1mexp(x)
}

#[no_mangle]
pub extern "C" fn log1pmx(x: c_double) -> c_double {
    r_ust_nmath::log1pmx(x)
}

#[no_mangle]
pub extern "C" fn logspace_add(logx: c_double, logy: c_double) -> c_double {
    r_ust_nmath::logspace_add(logx, logy)
}

#[no_mangle]
pub extern "C" fn logspace_sub(logx: c_double, logy: c_double) -> c_double {
    r_ust_nmath::logspace_sub(logx, logy)
}

#[no_mangle]
pub unsafe extern "C" fn logspace_sum(logx: *const c_double, n: c_int) -> c_double {
    if n < 0 || (n > 0 && logx.is_null()) {
        return c_double::NAN;
    }
    let values = std::slice::from_raw_parts(logx, n as usize);
    r_ust_nmath::logspace_sum(values)
}

#[no_mangle]
pub extern "C" fn pow1p(x: c_double, y: c_double) -> c_double {
    r_ust_nmath::pow1p(x, y)
}

#[no_mangle]
pub extern "C" fn gammafn(x: c_double) -> c_double {
    r_ust_nmath::gammafn(x)
}

#[no_mangle]
pub extern "C" fn lgammafn(x: c_double) -> c_double {
    r_ust_nmath::lgammafn(x)
}

#[no_mangle]
pub unsafe extern "C" fn lgammafn_sign(x: c_double, sgn: *mut c_int) -> c_double {
    let (value, sign) = r_ust_nmath::lgammafn_sign(x);
    if !sgn.is_null() {
        *sgn = sign;
    }
    value
}

#[no_mangle]
pub extern "C" fn lgamma1p(x: c_double) -> c_double {
    r_ust_nmath::lgamma1p(x)
}

#[no_mangle]
pub extern "C" fn beta(a: c_double, b: c_double) -> c_double {
    r_ust_nmath::beta(a, b)
}

#[no_mangle]
pub extern "C" fn lbeta(a: c_double, b: c_double) -> c_double {
    r_ust_nmath::lbeta(a, b)
}

#[no_mangle]
pub extern "C" fn choose(n: c_double, k: c_double) -> c_double {
    r_ust_nmath::choose(n, k)
}

#[no_mangle]
pub extern "C" fn lchoose(n: c_double, k: c_double) -> c_double {
    r_ust_nmath::lchoose(n, k)
}

#[no_mangle]
pub extern "C" fn dhyper(
    x: c_double,
    r: c_double,
    b: c_double,
    n: c_double,
    give_log: c_int,
) -> c_double {
    r_ust_nmath::dhyper(x, r, b, n, give_log != 0)
}

#[no_mangle]
pub extern "C" fn phyper(
    x: c_double,
    r: c_double,
    b: c_double,
    n: c_double,
    lower_tail: c_int,
    log_p: c_int,
) -> c_double {
    r_ust_nmath::phyper(x, r, b, n, lower_tail != 0, log_p != 0)
}

#[no_mangle]
pub extern "C" fn qhyper(
    p: c_double,
    r: c_double,
    b: c_double,
    n: c_double,
    lower_tail: c_int,
    log_p: c_int,
) -> c_double {
    r_ust_nmath::qhyper(p, r, b, n, lower_tail != 0, log_p != 0)
}

#[no_mangle]
pub extern "C" fn rhyper(r: c_double, b: c_double, n: c_double) -> c_double {
    r_ust_nmath::rhyper(r, b, n)
}

#[no_mangle]
pub unsafe extern "C" fn rmultinom(n: c_int, prob: *const c_double, k: c_int, out: *mut c_int) {
    if k < 0 || prob.is_null() || out.is_null() {
        return;
    }
    let probs = std::slice::from_raw_parts(prob, k as usize);
    let values = std::slice::from_raw_parts_mut(out, k as usize);
    r_ust_nmath::rmultinom(n, probs, values);
}

#[no_mangle]
pub extern "C" fn dsignrank(x: c_double, n: c_double, give_log: c_int) -> c_double {
    r_ust_nmath::dsignrank(x, n, give_log != 0)
}

#[no_mangle]
pub extern "C" fn psignrank(x: c_double, n: c_double, lower_tail: c_int, log_p: c_int) -> c_double {
    r_ust_nmath::psignrank(x, n, lower_tail != 0, log_p != 0)
}

#[no_mangle]
pub extern "C" fn qsignrank(p: c_double, n: c_double, lower_tail: c_int, log_p: c_int) -> c_double {
    r_ust_nmath::qsignrank(p, n, lower_tail != 0, log_p != 0)
}

#[no_mangle]
pub extern "C" fn rsignrank(n: c_double) -> c_double {
    r_ust_nmath::rsignrank(n)
}

#[no_mangle]
pub extern "C" fn signrank_free() {
    r_ust_nmath::signrank_free()
}

#[no_mangle]
pub extern "C" fn dwilcox(x: c_double, m: c_double, n: c_double, give_log: c_int) -> c_double {
    r_ust_nmath::dwilcox(x, m, n, give_log != 0)
}

#[no_mangle]
pub extern "C" fn pwilcox(
    x: c_double,
    m: c_double,
    n: c_double,
    lower_tail: c_int,
    log_p: c_int,
) -> c_double {
    r_ust_nmath::pwilcox(x, m, n, lower_tail != 0, log_p != 0)
}

#[no_mangle]
pub extern "C" fn qwilcox(
    p: c_double,
    m: c_double,
    n: c_double,
    lower_tail: c_int,
    log_p: c_int,
) -> c_double {
    r_ust_nmath::qwilcox(p, m, n, lower_tail != 0, log_p != 0)
}

#[no_mangle]
pub extern "C" fn rwilcox(m: c_double, n: c_double) -> c_double {
    r_ust_nmath::rwilcox(m, n)
}

#[no_mangle]
pub extern "C" fn wilcox_free() {
    r_ust_nmath::wilcox_free()
}

#[no_mangle]
pub extern "C" fn dnbeta(
    x: c_double,
    a: c_double,
    b: c_double,
    ncp: c_double,
    give_log: c_int,
) -> c_double {
    r_ust_nmath::dnbeta(x, a, b, ncp, give_log != 0)
}

#[no_mangle]
pub extern "C" fn pnbeta(
    x: c_double,
    a: c_double,
    b: c_double,
    ncp: c_double,
    lower_tail: c_int,
    log_p: c_int,
) -> c_double {
    r_ust_nmath::pnbeta(x, a, b, ncp, lower_tail != 0, log_p != 0)
}

#[no_mangle]
pub extern "C" fn qnbeta(
    p: c_double,
    a: c_double,
    b: c_double,
    ncp: c_double,
    lower_tail: c_int,
    log_p: c_int,
) -> c_double {
    r_ust_nmath::qnbeta(p, a, b, ncp, lower_tail != 0, log_p != 0)
}

#[no_mangle]
pub extern "C" fn pnchisq(
    x: c_double,
    df: c_double,
    ncp: c_double,
    lower_tail: c_int,
    log_p: c_int,
) -> c_double {
    r_ust_nmath::pnchisq(x, df, ncp, lower_tail != 0, log_p != 0)
}

#[no_mangle]
pub extern "C" fn qnchisq(
    p: c_double,
    df: c_double,
    ncp: c_double,
    lower_tail: c_int,
    log_p: c_int,
) -> c_double {
    r_ust_nmath::qnchisq(p, df, ncp, lower_tail != 0, log_p != 0)
}

#[no_mangle]
pub extern "C" fn dnf(
    x: c_double,
    df1: c_double,
    df2: c_double,
    ncp: c_double,
    give_log: c_int,
) -> c_double {
    r_ust_nmath::dnf(x, df1, df2, ncp, give_log != 0)
}

#[no_mangle]
pub extern "C" fn pnf(
    x: c_double,
    df1: c_double,
    df2: c_double,
    ncp: c_double,
    lower_tail: c_int,
    log_p: c_int,
) -> c_double {
    r_ust_nmath::pnf(x, df1, df2, ncp, lower_tail != 0, log_p != 0)
}

#[no_mangle]
pub extern "C" fn qnf(
    p: c_double,
    df1: c_double,
    df2: c_double,
    ncp: c_double,
    lower_tail: c_int,
    log_p: c_int,
) -> c_double {
    r_ust_nmath::qnf(p, df1, df2, ncp, lower_tail != 0, log_p != 0)
}

#[no_mangle]
pub extern "C" fn dnt(x: c_double, df: c_double, ncp: c_double, give_log: c_int) -> c_double {
    r_ust_nmath::dnt(x, df, ncp, give_log != 0)
}

#[no_mangle]
pub extern "C" fn pnt(
    x: c_double,
    df: c_double,
    ncp: c_double,
    lower_tail: c_int,
    log_p: c_int,
) -> c_double {
    r_ust_nmath::pnt(x, df, ncp, lower_tail != 0, log_p != 0)
}

#[no_mangle]
pub extern "C" fn qnt(
    p: c_double,
    df: c_double,
    ncp: c_double,
    lower_tail: c_int,
    log_p: c_int,
) -> c_double {
    r_ust_nmath::qnt(p, df, ncp, lower_tail != 0, log_p != 0)
}

#[no_mangle]
pub extern "C" fn ptukey(
    q: c_double,
    rr: c_double,
    cc: c_double,
    df: c_double,
    lower_tail: c_int,
    log_p: c_int,
) -> c_double {
    r_ust_nmath::ptukey(q, rr, cc, df, lower_tail != 0, log_p != 0)
}

#[no_mangle]
pub extern "C" fn qtukey(
    p: c_double,
    rr: c_double,
    cc: c_double,
    df: c_double,
    lower_tail: c_int,
    log_p: c_int,
) -> c_double {
    r_ust_nmath::qtukey(p, rr, cc, df, lower_tail != 0, log_p != 0)
}

#[no_mangle]
pub extern "C" fn fround(x: c_double, digits: c_double) -> c_double {
    r_ust_nmath::fround(x, digits)
}

#[no_mangle]
pub extern "C" fn fprec(x: c_double, digits: c_double) -> c_double {
    r_ust_nmath::fprec(x, digits)
}

#[no_mangle]
pub extern "C" fn digamma(x: c_double) -> c_double {
    r_ust_nmath::digamma(x)
}

#[no_mangle]
pub extern "C" fn trigamma(x: c_double) -> c_double {
    r_ust_nmath::trigamma(x)
}

#[no_mangle]
pub extern "C" fn tetragamma(x: c_double) -> c_double {
    r_ust_nmath::tetragamma(x)
}

#[no_mangle]
pub extern "C" fn pentagamma(x: c_double) -> c_double {
    r_ust_nmath::pentagamma(x)
}

#[no_mangle]
pub extern "C" fn psigamma(x: c_double, deriv: c_double) -> c_double {
    r_ust_nmath::psigamma(x, deriv)
}

#[no_mangle]
pub unsafe extern "C" fn dpsifn(
    x: c_double,
    n: c_int,
    kode: c_int,
    m: c_int,
    ans: *mut c_double,
    nz: *mut c_int,
    ierr: *mut c_int,
) {
    if m < 0 || ans.is_null() {
        if !ierr.is_null() {
            *ierr = 1;
        }
        return;
    }
    let (values, nz_value, ierr_value) = r_ust_nmath::dpsifn(x, n, kode, m);
    let out = std::slice::from_raw_parts_mut(ans, m as usize);
    for (slot, value) in out.iter_mut().zip(values.iter()) {
        *slot = *value;
    }
    if !nz.is_null() {
        *nz = nz_value;
    }
    if !ierr.is_null() {
        *ierr = ierr_value;
    }
}

#[no_mangle]
pub extern "C" fn bessel_j(x: c_double, alpha: c_double) -> c_double {
    r_ust_nmath::bessel_j(x, alpha)
}

#[no_mangle]
pub unsafe extern "C" fn bessel_j_ex(
    x: c_double,
    alpha: c_double,
    work: *mut c_double,
) -> c_double {
    let slot = if work.is_null() {
        None
    } else {
        Some(&mut *work)
    };
    r_ust_nmath::bessel_j_ex(x, alpha, slot)
}

#[no_mangle]
pub extern "C" fn bessel_y(x: c_double, alpha: c_double) -> c_double {
    r_ust_nmath::bessel_y(x, alpha)
}

#[no_mangle]
pub unsafe extern "C" fn bessel_y_ex(
    x: c_double,
    alpha: c_double,
    work: *mut c_double,
) -> c_double {
    let slot = if work.is_null() {
        None
    } else {
        Some(&mut *work)
    };
    r_ust_nmath::bessel_y_ex(x, alpha, slot)
}

#[no_mangle]
pub extern "C" fn bessel_i(x: c_double, alpha: c_double, expo: c_double) -> c_double {
    r_ust_nmath::bessel_i(x, alpha, expo)
}

#[no_mangle]
pub unsafe extern "C" fn bessel_i_ex(
    x: c_double,
    alpha: c_double,
    expo: c_double,
    work: *mut c_double,
) -> c_double {
    let slot = if work.is_null() {
        None
    } else {
        Some(&mut *work)
    };
    r_ust_nmath::bessel_i_ex(x, alpha, expo, slot)
}

#[no_mangle]
pub extern "C" fn bessel_k(x: c_double, alpha: c_double, expo: c_double) -> c_double {
    r_ust_nmath::bessel_k(x, alpha, expo)
}

#[no_mangle]
pub unsafe extern "C" fn bessel_k_ex(
    x: c_double,
    alpha: c_double,
    expo: c_double,
    work: *mut c_double,
) -> c_double {
    let slot = if work.is_null() {
        None
    } else {
        Some(&mut *work)
    };
    r_ust_nmath::bessel_k_ex(x, alpha, expo, slot)
}
