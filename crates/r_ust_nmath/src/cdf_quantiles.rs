use crate::elementary::{d_val, dt_0, dt_1, q_p01_boundaries, q_p01_check};
use crate::special::{lbeta, lgammafn};

const EPS: f64 = 1e-14;
const FPMIN: f64 = 1e-300;

pub fn pgamma(x: f64, shape: f64, scale: f64, lower_tail: bool, log_p: bool) -> f64 {
    if x.is_nan() || shape.is_nan() || scale.is_nan() {
        return x + shape + scale;
    }
    if shape <= 0.0 || scale <= 0.0 {
        return f64::NAN;
    }
    if x <= 0.0 {
        return dt_0(lower_tail, log_p);
    }
    if !x.is_finite() {
        return dt_1(lower_tail, log_p);
    }
    let p = regularized_gamma_p(shape, x / scale).clamp(0.0, 1.0);
    tail_result(p, lower_tail, log_p)
}

pub fn qgamma(p: f64, shape: f64, scale: f64, lower_tail: bool, log_p: bool) -> f64 {
    if p.is_nan() || shape.is_nan() || scale.is_nan() {
        return p + shape + scale;
    }
    if shape <= 0.0 || scale <= 0.0 || q_p01_check(p, log_p) {
        return f64::NAN;
    }
    let target = match probability_from_args(p, lower_tail, log_p) {
        Some(value) => value,
        None => return f64::NAN,
    };
    if target <= 0.0 {
        return 0.0;
    }
    if target >= 1.0 {
        return f64::INFINITY;
    }
    let mut lo = 0.0;
    let mut hi = (shape * scale).max(scale).max(1.0);
    while pgamma(hi, shape, scale, true, false) < target {
        hi *= 2.0;
        if !hi.is_finite() {
            return f64::INFINITY;
        }
    }
    for _ in 0..160 {
        let mid = 0.5 * (lo + hi);
        if pgamma(mid, shape, scale, true, false) < target {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    0.5 * (lo + hi)
}

pub fn pchisq(x: f64, df: f64, lower_tail: bool, log_p: bool) -> f64 {
    pgamma(x, df / 2.0, 2.0, lower_tail, log_p)
}

pub fn qchisq(p: f64, df: f64, lower_tail: bool, log_p: bool) -> f64 {
    qgamma(p, df / 2.0, 2.0, lower_tail, log_p)
}

pub fn pbeta(x: f64, a: f64, b: f64, lower_tail: bool, log_p: bool) -> f64 {
    if x.is_nan() || a.is_nan() || b.is_nan() {
        return x + a + b;
    }
    if a <= 0.0 || b <= 0.0 {
        return f64::NAN;
    }
    if x <= 0.0 {
        return dt_0(lower_tail, log_p);
    }
    if x >= 1.0 {
        return dt_1(lower_tail, log_p);
    }
    let p = regularized_beta(x, a, b).clamp(0.0, 1.0);
    tail_result(p, lower_tail, log_p)
}

pub fn qbeta(p: f64, a: f64, b: f64, lower_tail: bool, log_p: bool) -> f64 {
    if p.is_nan() || a.is_nan() || b.is_nan() {
        return p + a + b;
    }
    if a <= 0.0 || b <= 0.0 || q_p01_check(p, log_p) {
        return f64::NAN;
    }
    match q_p01_boundaries(p, lower_tail, log_p, 0.0, 1.0) {
        Ok(Some(boundary)) => return boundary,
        Ok(None) => {}
        Err(value) => return value,
    }
    let target = match probability_from_args(p, lower_tail, log_p) {
        Some(value) => value,
        None => return f64::NAN,
    };
    let mut lo = 0.0;
    let mut hi = 1.0;
    for _ in 0..160 {
        let mid = 0.5 * (lo + hi);
        if pbeta(mid, a, b, true, false) < target {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    0.5 * (lo + hi)
}

pub fn pf(x: f64, m: f64, n: f64, lower_tail: bool, log_p: bool) -> f64 {
    if x.is_nan() || m.is_nan() || n.is_nan() {
        return x + m + n;
    }
    if m <= 0.0 || n <= 0.0 {
        return f64::NAN;
    }
    if x <= 0.0 {
        return dt_0(lower_tail, log_p);
    }
    if !x.is_finite() {
        return dt_1(lower_tail, log_p);
    }
    let z = (m * x) / (n + m * x);
    pbeta(z, m / 2.0, n / 2.0, lower_tail, log_p)
}

pub fn qf(p: f64, m: f64, n: f64, lower_tail: bool, log_p: bool) -> f64 {
    if p.is_nan() || m.is_nan() || n.is_nan() {
        return p + m + n;
    }
    if m <= 0.0 || n <= 0.0 {
        return f64::NAN;
    }
    let z = qbeta(p, m / 2.0, n / 2.0, lower_tail, log_p);
    if z >= 1.0 {
        f64::INFINITY
    } else {
        n * z / (m * (1.0 - z))
    }
}

pub fn pt(x: f64, df: f64, lower_tail: bool, log_p: bool) -> f64 {
    if x.is_nan() || df.is_nan() {
        return x + df;
    }
    if df <= 0.0 {
        return f64::NAN;
    }
    if !x.is_finite() {
        return if x < 0.0 {
            dt_0(lower_tail, log_p)
        } else {
            dt_1(lower_tail, log_p)
        };
    }
    if !df.is_finite() {
        return crate::pnorm(x, 0.0, 1.0, lower_tail, log_p);
    }
    let z = df / (df + x * x);
    let ib = regularized_beta(z, df / 2.0, 0.5).clamp(0.0, 1.0);
    let p = if x >= 0.0 { 1.0 - 0.5 * ib } else { 0.5 * ib };
    tail_result(p, lower_tail, log_p)
}

pub fn qt(p: f64, df: f64, lower_tail: bool, log_p: bool) -> f64 {
    if p.is_nan() || df.is_nan() {
        return p + df;
    }
    if df <= 0.0 || q_p01_check(p, log_p) {
        return f64::NAN;
    }
    match q_p01_boundaries(p, lower_tail, log_p, f64::NEG_INFINITY, f64::INFINITY) {
        Ok(Some(boundary)) => return boundary,
        Ok(None) => {}
        Err(value) => return value,
    }
    let target = match probability_from_args(p, lower_tail, log_p) {
        Some(value) => value,
        None => return f64::NAN,
    };
    if !df.is_finite() {
        return crate::qnorm(target, 0.0, 1.0, true, false);
    }
    let mut lo = -1.0;
    let mut hi = 1.0;
    while pt(lo, df, true, false) > target {
        lo *= 2.0;
    }
    while pt(hi, df, true, false) < target {
        hi *= 2.0;
    }
    for _ in 0..160 {
        let mid = 0.5 * (lo + hi);
        if pt(mid, df, true, false) < target {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    0.5 * (lo + hi)
}

pub fn pnbinom(x: f64, size: f64, prob: f64, lower_tail: bool, log_p: bool) -> f64 {
    if x.is_nan() || size.is_nan() || prob.is_nan() {
        return x + size + prob;
    }
    if !size.is_finite() || !prob.is_finite() || size < 0.0 || prob <= 0.0 || prob > 1.0 {
        return f64::NAN;
    }
    if size == 0.0 {
        return if x >= 0.0 {
            dt_1(lower_tail, log_p)
        } else {
            dt_0(lower_tail, log_p)
        };
    }
    if x < 0.0 {
        return dt_0(lower_tail, log_p);
    }
    if !x.is_finite() {
        return dt_1(lower_tail, log_p);
    }
    pbeta(prob, size, (x + 1e-7).floor() + 1.0, lower_tail, log_p)
}

pub fn pnbinom_mu(x: f64, size: f64, mu: f64, lower_tail: bool, log_p: bool) -> f64 {
    if x.is_nan() || size.is_nan() || mu.is_nan() {
        return x + size + mu;
    }
    if !mu.is_finite() || size < 0.0 || mu < 0.0 {
        return f64::NAN;
    }
    if size == 0.0 {
        return if x >= 0.0 {
            dt_1(lower_tail, log_p)
        } else {
            dt_0(lower_tail, log_p)
        };
    }
    if x < 0.0 {
        return dt_0(lower_tail, log_p);
    }
    if !x.is_finite() {
        return dt_1(lower_tail, log_p);
    }
    if !size.is_finite() {
        return crate::ppois(x, mu, lower_tail, log_p);
    }
    pbeta(
        size / (size + mu),
        size,
        (x + 1e-7).floor() + 1.0,
        lower_tail,
        log_p,
    )
}

pub fn qnbinom(p: f64, size: f64, prob: f64, lower_tail: bool, log_p: bool) -> f64 {
    if p.is_nan() || size.is_nan() || prob.is_nan() {
        return p + size + prob;
    }
    if size < 0.0 || prob <= 0.0 || prob > 1.0 || q_p01_check(p, log_p) {
        return f64::NAN;
    }
    discrete_quantile(p, lower_tail, log_p, |x| {
        pnbinom(x, size, prob, true, false)
    })
}

pub fn qnbinom_mu(p: f64, size: f64, mu: f64, lower_tail: bool, log_p: bool) -> f64 {
    if p.is_nan() || size.is_nan() || mu.is_nan() {
        return p + size + mu;
    }
    if size < 0.0 || mu < 0.0 || q_p01_check(p, log_p) {
        return f64::NAN;
    }
    discrete_quantile(p, lower_tail, log_p, |x| {
        pnbinom_mu(x, size, mu, true, false)
    })
}

fn regularized_gamma_p(a: f64, x: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    if x < a + 1.0 {
        let gln = lgammafn(a);
        let mut ap = a;
        let mut del = 1.0 / a;
        let mut sum = del;
        for _ in 0..10_000 {
            ap += 1.0;
            del *= x / ap;
            sum += del;
            if del.abs() < sum.abs() * EPS {
                break;
            }
        }
        sum * (-x + a * x.ln() - gln).exp()
    } else {
        1.0 - regularized_gamma_q_cf(a, x)
    }
}

fn regularized_gamma_q_cf(a: f64, x: f64) -> f64 {
    let gln = lgammafn(a);
    let mut b = x + 1.0 - a;
    let mut c = 1.0 / FPMIN;
    let mut d = 1.0 / b.max(FPMIN);
    let mut h = d;
    for i in 1..10_000 {
        let fi = i as f64;
        let an = -fi * (fi - a);
        b += 2.0;
        d = an * d + b;
        if d.abs() < FPMIN {
            d = FPMIN;
        }
        c = b + an / c;
        if c.abs() < FPMIN {
            c = FPMIN;
        }
        d = 1.0 / d;
        let del = d * c;
        h *= del;
        if (del - 1.0).abs() < EPS {
            break;
        }
    }
    (-x + a * x.ln() - gln).exp() * h
}

fn regularized_beta(x: f64, a: f64, b: f64) -> f64 {
    let bt = if x == 0.0 || x == 1.0 {
        0.0
    } else {
        (a * x.ln() + b * (-x).ln_1p() - lbeta(a, b)).exp()
    };
    if x < (a + 1.0) / (a + b + 2.0) {
        bt * betacf(a, b, x) / a
    } else {
        1.0 - bt * betacf(b, a, 1.0 - x) / b
    }
}

fn betacf(a: f64, b: f64, x: f64) -> f64 {
    let qab = a + b;
    let qap = a + 1.0;
    let qam = a - 1.0;
    let mut c = 1.0;
    let mut d = 1.0 - qab * x / qap;
    if d.abs() < FPMIN {
        d = FPMIN;
    }
    d = 1.0 / d;
    let mut h = d;
    for m in 1..10_000 {
        let m2 = 2 * m;
        let fm = m as f64;
        let fm2 = m2 as f64;
        let mut aa = fm * (b - fm) * x / ((qam + fm2) * (a + fm2));
        d = 1.0 + aa * d;
        if d.abs() < FPMIN {
            d = FPMIN;
        }
        c = 1.0 + aa / c;
        if c.abs() < FPMIN {
            c = FPMIN;
        }
        d = 1.0 / d;
        h *= d * c;

        aa = -(a + fm) * (qab + fm) * x / ((a + fm2) * (qap + fm2));
        d = 1.0 + aa * d;
        if d.abs() < FPMIN {
            d = FPMIN;
        }
        c = 1.0 + aa / c;
        if c.abs() < FPMIN {
            c = FPMIN;
        }
        d = 1.0 / d;
        let del = d * c;
        h *= del;
        if (del - 1.0).abs() < EPS {
            break;
        }
    }
    h
}

fn tail_result(lower_prob: f64, lower_tail: bool, log_p: bool) -> f64 {
    let prob = if lower_tail {
        lower_prob
    } else {
        1.0 - lower_prob
    };
    d_val(prob.clamp(0.0, 1.0), log_p)
}

fn probability_from_args(p: f64, lower_tail: bool, log_p: bool) -> Option<f64> {
    if q_p01_check(p, log_p) {
        return None;
    }
    Some(if log_p {
        if lower_tail {
            p.exp()
        } else {
            -p.exp_m1()
        }
    } else if lower_tail {
        p
    } else {
        1.0 - p
    })
}

fn discrete_quantile<F>(p: f64, lower_tail: bool, log_p: bool, cdf: F) -> f64
where
    F: Fn(f64) -> f64,
{
    let target = match probability_from_args(p, lower_tail, log_p) {
        Some(value) => value,
        None => return f64::NAN,
    };
    if target <= 0.0 {
        return 0.0;
    }
    if target >= 1.0 {
        return f64::INFINITY;
    }
    let mut hi = 1.0;
    while cdf(hi) < target {
        hi *= 2.0;
        if !hi.is_finite() {
            return f64::INFINITY;
        }
    }
    let mut lo = 0.0;
    while lo < hi {
        let mid = ((lo + hi) / 2.0).floor();
        if mid == lo {
            return if cdf(lo) >= target { lo } else { hi };
        }
        if cdf(mid) < target {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    lo
}
