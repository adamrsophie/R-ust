use crate::elementary::{
    d_0, d_1, d_exp, dt_0, dt_1, dt_clog, fmax2, force_int, nonint, q_p01_boundaries, q_p01_check,
};
use crate::special::{bd0, lgammafn, stirlerr};

const M_LN_2PI: f64 = 1.837_877_066_409_345_483_560_659_472_811;

pub fn dbinom_raw(x: f64, mut n: f64, p: f64, q: f64, give_log: bool) -> f64 {
    if p == 0.0 {
        return if x == 0.0 {
            d_1(give_log)
        } else {
            d_0(give_log)
        };
    }
    if q == 0.0 {
        return if x == n { d_1(give_log) } else { d_0(give_log) };
    }
    if x == 0.0 {
        if n == 0.0 {
            return d_1(give_log);
        }
        return if p > q {
            if give_log {
                n * q.ln()
            } else {
                q.powf(n)
            }
        } else if give_log {
            n * (-p).ln_1p()
        } else {
            crate::pow1p(-p, n)
        };
    }
    if x == n {
        return if p > q {
            if give_log {
                n * (-q).ln_1p()
            } else {
                crate::pow1p(-q, n)
            }
        } else if give_log {
            n * p.ln()
        } else {
            p.powf(n)
        };
    }
    if x < 0.0 || x > n {
        return d_0(give_log);
    }
    if !n.is_finite() {
        if x.is_finite() {
            return d_0(give_log);
        }
        n = f64::MAX;
    }
    let lc = stirlerr(n) - stirlerr(x) - stirlerr(n - x) - bd0(x, n * p) - bd0(n - x, n * q);
    let lf = M_LN_2PI + x.ln() + (-x / n).ln_1p();
    d_exp(lc - 0.5 * lf, give_log)
}

pub fn dbinom(x: f64, n: f64, p: f64, give_log: bool) -> f64 {
    if x.is_nan() || n.is_nan() || p.is_nan() {
        return x + n + p;
    }
    if !(0.0..=1.0).contains(&p) || n < 0.0 || nonint(n) {
        return f64::NAN;
    }
    if nonint(x) {
        return d_0(give_log);
    }
    if x < 0.0 || !x.is_finite() {
        return d_0(give_log);
    }
    dbinom_raw(force_int(x), force_int(n), p, 1.0 - p, give_log)
}

pub fn dpois_raw(x: f64, lambda: f64, give_log: bool) -> f64 {
    if lambda == 0.0 {
        return if x == 0.0 {
            d_1(give_log)
        } else {
            d_0(give_log)
        };
    }
    if !lambda.is_finite() || x < 0.0 {
        return d_0(give_log);
    }
    if x <= lambda * f64::MIN_POSITIVE {
        return d_exp(-lambda, give_log);
    }
    if lambda < x * f64::MIN_POSITIVE {
        if !x.is_finite() {
            return d_0(give_log);
        }
        return d_exp(-lambda + x * lambda.ln() - lgammafn(x + 1.0), give_log);
    }
    d_exp(-lambda + x * lambda.ln() - lgammafn(x + 1.0), give_log)
}

pub fn dpois(x: f64, lambda: f64, give_log: bool) -> f64 {
    if x.is_nan() || lambda.is_nan() {
        return x + lambda;
    }
    if lambda < 0.0 {
        return f64::NAN;
    }
    if nonint(x) {
        return d_0(give_log);
    }
    if x < 0.0 || !x.is_finite() {
        return d_0(give_log);
    }
    dpois_raw(force_int(x), lambda, give_log)
}

pub fn ppois(x: f64, lambda: f64, lower_tail: bool, log_p: bool) -> f64 {
    if x.is_nan() || lambda.is_nan() {
        return x + lambda;
    }
    if lambda < 0.0 {
        return f64::NAN;
    }
    if x < 0.0 {
        return dt_0(lower_tail, log_p);
    }
    if lambda == 0.0 || !x.is_finite() {
        return dt_1(lower_tail, log_p);
    }
    let k = (x + 1e-7).floor() as u64;
    let mut term = (-lambda).exp();
    let mut sum = term;
    for i in 1..=k {
        term *= lambda / i as f64;
        sum += term;
        if term == 0.0 {
            break;
        }
    }
    let prob = sum.clamp(0.0, 1.0);
    if lower_tail {
        if log_p {
            prob.ln()
        } else {
            prob
        }
    } else {
        let upper = (1.0 - prob).max(0.0);
        if log_p {
            upper.ln()
        } else {
            upper
        }
    }
}

pub fn qpois(p: f64, lambda: f64, lower_tail: bool, log_p: bool) -> f64 {
    if p.is_nan() || lambda.is_nan() {
        return p + lambda;
    }
    if !lambda.is_finite() || lambda < 0.0 || q_p01_check(p, log_p) {
        return f64::NAN;
    }
    if lambda == 0.0 {
        return 0.0;
    }
    match q_p01_boundaries(p, lower_tail, log_p, 0.0, f64::INFINITY) {
        Ok(Some(boundary)) => return boundary,
        Ok(None) => {}
        Err(value) => return value,
    }
    let target = if log_p {
        if lower_tail {
            p.exp()
        } else {
            -p.exp_m1()
        }
    } else if lower_tail {
        p
    } else {
        1.0 - p
    };
    let mut k = 0.0;
    let mut term = (-lambda).exp();
    let mut sum = term;
    while sum < target && k < lambda + 20.0 * lambda.sqrt() + 1000.0 {
        k += 1.0;
        term *= lambda / k;
        sum += term;
    }
    k
}

pub fn rpois_from_state(state: &mut crate::RngState, lambda: f64) -> f64 {
    if !lambda.is_finite() || lambda < 0.0 {
        return f64::NAN;
    }
    if lambda <= 0.0 {
        return 0.0;
    }
    if lambda < 30.0 {
        let limit = (-lambda).exp();
        let mut p = 1.0;
        let mut k = 0.0;
        loop {
            k += 1.0;
            p *= state.unif_rand();
            if p <= limit {
                return k - 1.0;
            }
        }
    }
    let normal = state.rnorm_inversion(lambda, lambda.sqrt()).round();
    normal.max(0.0)
}

pub fn dgeom(x: f64, p: f64, give_log: bool) -> f64 {
    if x.is_nan() || p.is_nan() {
        return x + p;
    }
    if p <= 0.0 || p > 1.0 {
        return f64::NAN;
    }
    if nonint(x) || x < 0.0 || !x.is_finite() {
        return d_0(give_log);
    }
    let x = force_int(x);
    if give_log {
        p.ln() + x * (-p).ln_1p()
    } else {
        p * (1.0 - p).powf(x)
    }
}

pub fn pgeom(x: f64, p: f64, lower_tail: bool, log_p: bool) -> f64 {
    if x.is_nan() || p.is_nan() {
        return x + p;
    }
    if p <= 0.0 || p > 1.0 {
        return f64::NAN;
    }
    if x < 0.0 {
        return dt_0(lower_tail, log_p);
    }
    if !x.is_finite() {
        return dt_1(lower_tail, log_p);
    }
    let x = (x + 1e-7).floor();
    if p == 1.0 {
        let value: f64 = if lower_tail { 1.0 } else { 0.0 };
        return if log_p { value.ln() } else { value };
    }
    let x = (-p).ln_1p() * (x + 1.0);
    if log_p {
        dt_clog(x, lower_tail, true)
    } else if lower_tail {
        -x.exp_m1()
    } else {
        x.exp()
    }
}

pub fn qgeom(p: f64, prob: f64, lower_tail: bool, log_p: bool) -> f64 {
    if p.is_nan() || prob.is_nan() {
        return p + prob;
    }
    if prob <= 0.0 || prob > 1.0 || q_p01_check(p, log_p) {
        return f64::NAN;
    }
    if prob == 1.0 {
        return 0.0;
    }
    match q_p01_boundaries(p, lower_tail, log_p, 0.0, f64::INFINITY) {
        Ok(Some(boundary)) => return boundary,
        Ok(None) => {}
        Err(value) => return value,
    }
    fmax2(
        0.0,
        (dt_clog(p, lower_tail, log_p) / (-prob).ln_1p() - 1.0 - 1e-12).ceil(),
    )
}

pub fn rgeom_from_state(state: &mut crate::RngState, p: f64) -> f64 {
    if !p.is_finite() || p <= 0.0 || p > 1.0 {
        return f64::NAN;
    }
    let lambda = state.exp_rand() * ((1.0 - p) / p);
    rpois_from_state(state, lambda)
}

pub fn pbinom(x: f64, n: f64, p: f64, lower_tail: bool, log_p: bool) -> f64 {
    if x.is_nan() || n.is_nan() || p.is_nan() {
        return x + n + p;
    }
    if !(0.0..=1.0).contains(&p) || n < 0.0 || nonint(n) {
        return f64::NAN;
    }
    if x < 0.0 {
        return dt_0(lower_tail, log_p);
    }
    let n = force_int(n) as u64;
    if x >= n as f64 {
        return dt_1(lower_tail, log_p);
    }
    let xmax = (x + 1e-7).floor() as u64;
    let mut sum = 0.0;
    for k in 0..=xmax {
        sum += dbinom(k as f64, n as f64, p, false);
    }
    let prob = sum.clamp(0.0, 1.0);
    if lower_tail {
        if log_p {
            prob.ln()
        } else {
            prob
        }
    } else {
        let upper = (1.0 - prob).max(0.0);
        if log_p {
            upper.ln()
        } else {
            upper
        }
    }
}

pub fn qbinom(p: f64, n: f64, prob: f64, lower_tail: bool, log_p: bool) -> f64 {
    if p.is_nan() || n.is_nan() || prob.is_nan() {
        return p + n + prob;
    }
    if !(0.0..=1.0).contains(&prob) || n < 0.0 || nonint(n) || q_p01_check(p, log_p) {
        return f64::NAN;
    }
    let n_int = force_int(n) as u64;
    match q_p01_boundaries(p, lower_tail, log_p, 0.0, n) {
        Ok(Some(boundary)) => return boundary,
        Ok(None) => {}
        Err(value) => return value,
    }
    let target = if log_p {
        if lower_tail {
            p.exp()
        } else {
            -p.exp_m1()
        }
    } else if lower_tail {
        p
    } else {
        1.0 - p
    };
    let mut sum = 0.0;
    for k in 0..=n_int {
        sum += dbinom(k as f64, n, prob, false);
        if sum >= target {
            return k as f64;
        }
    }
    n
}

pub fn rbinom_from_state(state: &mut crate::RngState, n: f64, p: f64) -> f64 {
    if n.is_nan() || p.is_nan() || !(0.0..=1.0).contains(&p) || n < 0.0 || nonint(n) {
        return f64::NAN;
    }
    let n = force_int(n) as u64;
    let mut count = 0.0;
    for _ in 0..n {
        if state.unif_rand() < p {
            count += 1.0;
        }
    }
    count
}
