use crate::elementary::{d_0, d_exp, d_val, dt_0, dt_1, q_p01_boundaries, q_p01_check};

const POIS_EPS: f64 = 1e-13;

pub fn pnchisq(x: f64, df: f64, ncp: f64, lower_tail: bool, log_p: bool) -> f64 {
    if x.is_nan() || df.is_nan() || ncp.is_nan() {
        return x + df + ncp;
    }
    if df < 0.0 || ncp < 0.0 || !df.is_finite() || !ncp.is_finite() {
        return f64::NAN;
    }
    if x <= 0.0 {
        return dt_0(lower_tail, log_p);
    }
    if !x.is_finite() {
        return dt_1(lower_tail, log_p);
    }
    if ncp == 0.0 {
        return crate::pchisq(x, df, lower_tail, log_p);
    }
    let theta = 0.5 * ncp;
    let p = poisson_mixture_prob(theta, |k| crate::pchisq(x, df + 2.0 * k, true, false));
    tail_result(p, lower_tail, log_p)
}

pub fn qnchisq(p: f64, df: f64, ncp: f64, lower_tail: bool, log_p: bool) -> f64 {
    if p.is_nan() || df.is_nan() || ncp.is_nan() {
        return p + df + ncp;
    }
    if df < 0.0 || ncp < 0.0 || q_p01_check(p, log_p) {
        return f64::NAN;
    }
    match q_p01_boundaries(p, lower_tail, log_p, 0.0, f64::INFINITY) {
        Ok(Some(boundary)) => return boundary,
        Ok(None) => {}
        Err(value) => return value,
    }
    let target = probability_from_args(p, lower_tail, log_p);
    let lo = 0.0;
    let mut hi = (df + ncp).max(1.0);
    while pnchisq(hi, df, ncp, true, false) < target {
        hi *= 2.0;
        if !hi.is_finite() {
            return f64::INFINITY;
        }
    }
    bisect(lo, hi, |x| pnchisq(x, df, ncp, true, false), target)
}

pub fn dnbeta(x: f64, a: f64, b: f64, ncp: f64, give_log: bool) -> f64 {
    if x.is_nan() || a.is_nan() || b.is_nan() || ncp.is_nan() {
        return x + a + b + ncp;
    }
    if a <= 0.0 || b <= 0.0 || ncp < 0.0 || !ncp.is_finite() {
        return f64::NAN;
    }
    if !(0.0..=1.0).contains(&x) {
        return d_0(give_log);
    }
    if ncp == 0.0 {
        return crate::dbeta(x, a, b, give_log);
    }
    let theta = 0.5 * ncp;
    let dens = poisson_mixture_raw(theta, |k| crate::dbeta(x, a + k, b, false));
    d_exp(dens.ln(), give_log)
}

pub fn pnbeta(x: f64, a: f64, b: f64, ncp: f64, lower_tail: bool, log_p: bool) -> f64 {
    if x.is_nan() || a.is_nan() || b.is_nan() || ncp.is_nan() {
        return x + a + b + ncp;
    }
    if a <= 0.0 || b <= 0.0 || ncp < 0.0 || !ncp.is_finite() {
        return f64::NAN;
    }
    if x <= 0.0 {
        return dt_0(lower_tail, log_p);
    }
    if x >= 1.0 {
        return dt_1(lower_tail, log_p);
    }
    if ncp == 0.0 {
        return crate::pbeta(x, a, b, lower_tail, log_p);
    }
    let theta = 0.5 * ncp;
    let p = poisson_mixture_prob(theta, |k| crate::pbeta(x, a + k, b, true, false));
    tail_result(p, lower_tail, log_p)
}

pub fn qnbeta(p: f64, a: f64, b: f64, ncp: f64, lower_tail: bool, log_p: bool) -> f64 {
    if p.is_nan() || a.is_nan() || b.is_nan() || ncp.is_nan() {
        return p + a + b + ncp;
    }
    if a <= 0.0 || b <= 0.0 || ncp < 0.0 || q_p01_check(p, log_p) {
        return f64::NAN;
    }
    match q_p01_boundaries(p, lower_tail, log_p, 0.0, 1.0) {
        Ok(Some(boundary)) => return boundary,
        Ok(None) => {}
        Err(value) => return value,
    }
    let target = probability_from_args(p, lower_tail, log_p);
    bisect(0.0, 1.0, |x| pnbeta(x, a, b, ncp, true, false), target)
}

pub fn dnf(x: f64, df1: f64, df2: f64, ncp: f64, give_log: bool) -> f64 {
    if x.is_nan() || df1.is_nan() || df2.is_nan() || ncp.is_nan() {
        return x + df1 + df2 + ncp;
    }
    if df1 <= 0.0 || df2 <= 0.0 || ncp < 0.0 {
        return f64::NAN;
    }
    if x <= 0.0 {
        return d_0(give_log);
    }
    if ncp == 0.0 {
        return crate::df(x, df1, df2, give_log);
    }
    let y = df1 * x / df2;
    let z = y / (1.0 + y);
    let dens = dnbeta(z, df1 / 2.0, df2 / 2.0, ncp, false) * df1 / df2 / (1.0 + y).powi(2);
    d_exp(dens.ln(), give_log)
}

pub fn pnf(x: f64, df1: f64, df2: f64, ncp: f64, lower_tail: bool, log_p: bool) -> f64 {
    if x.is_nan() || df1.is_nan() || df2.is_nan() || ncp.is_nan() {
        return x + df1 + df2 + ncp;
    }
    if df1 <= 0.0 || df2 <= 0.0 || ncp < 0.0 {
        return f64::NAN;
    }
    if x <= 0.0 {
        return dt_0(lower_tail, log_p);
    }
    if !x.is_finite() {
        return dt_1(lower_tail, log_p);
    }
    if ncp == 0.0 {
        return crate::pf(x, df1, df2, lower_tail, log_p);
    }
    if !df2.is_finite() {
        return pnchisq(x * df1, df1, ncp, lower_tail, log_p);
    }
    let y = df1 * x / df2;
    pnbeta(y / (1.0 + y), df1 / 2.0, df2 / 2.0, ncp, lower_tail, log_p)
}

pub fn qnf(p: f64, df1: f64, df2: f64, ncp: f64, lower_tail: bool, log_p: bool) -> f64 {
    if p.is_nan() || df1.is_nan() || df2.is_nan() || ncp.is_nan() {
        return p + df1 + df2 + ncp;
    }
    if df1 <= 0.0 || df2 <= 0.0 || ncp < 0.0 || q_p01_check(p, log_p) {
        return f64::NAN;
    }
    match q_p01_boundaries(p, lower_tail, log_p, 0.0, f64::INFINITY) {
        Ok(Some(boundary)) => return boundary,
        Ok(None) => {}
        Err(value) => return value,
    }
    if ncp == 0.0 {
        return crate::qf(p, df1, df2, lower_tail, log_p);
    }
    let z = qnbeta(p, df1 / 2.0, df2 / 2.0, ncp, lower_tail, log_p);
    df2 * z / (df1 * (1.0 - z))
}

pub fn pnt(x: f64, df: f64, ncp: f64, lower_tail: bool, log_p: bool) -> f64 {
    if x.is_nan() || df.is_nan() || ncp.is_nan() {
        return x + df + ncp;
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
    if ncp == 0.0 {
        return crate::pt(x, df, lower_tail, log_p);
    }
    if !df.is_finite() {
        return crate::pnorm(x, ncp, 1.0, lower_tail, log_p);
    }
    let p = integrate_noncentral_t(x, df, ncp).clamp(0.0, 1.0);
    tail_result(p, lower_tail, log_p)
}

pub fn dnt(x: f64, df: f64, ncp: f64, give_log: bool) -> f64 {
    if x.is_nan() || df.is_nan() || ncp.is_nan() {
        return x + df + ncp;
    }
    if df <= 0.0 {
        return f64::NAN;
    }
    if ncp == 0.0 {
        return crate::dt(x, df, give_log);
    }
    let h = 1e-5 * (1.0 + x.abs());
    let dens = (pnt(x + h, df, ncp, true, false) - pnt(x - h, df, ncp, true, false)) / (2.0 * h);
    d_exp(dens.max(0.0).ln(), give_log)
}

pub fn qnt(p: f64, df: f64, ncp: f64, lower_tail: bool, log_p: bool) -> f64 {
    if p.is_nan() || df.is_nan() || ncp.is_nan() {
        return p + df + ncp;
    }
    if df <= 0.0 || q_p01_check(p, log_p) {
        return f64::NAN;
    }
    match q_p01_boundaries(p, lower_tail, log_p, f64::NEG_INFINITY, f64::INFINITY) {
        Ok(Some(boundary)) => return boundary,
        Ok(None) => {}
        Err(value) => return value,
    }
    let target = probability_from_args(p, lower_tail, log_p);
    let mut lo = -1.0;
    let mut hi = 1.0;
    while pnt(lo, df, ncp, true, false) > target {
        lo *= 2.0;
    }
    while pnt(hi, df, ncp, true, false) < target {
        hi *= 2.0;
    }
    bisect(lo, hi, |x| pnt(x, df, ncp, true, false), target)
}

pub fn ptukey(q: f64, rr: f64, cc: f64, df: f64, lower_tail: bool, log_p: bool) -> f64 {
    if q.is_nan() || rr.is_nan() || cc.is_nan() || df.is_nan() {
        return q + rr + cc + df;
    }
    if rr < 1.0 || cc < 2.0 || df <= 0.0 {
        return f64::NAN;
    }
    if q <= 0.0 {
        return dt_0(lower_tail, log_p);
    }
    if !q.is_finite() {
        return dt_1(lower_tail, log_p);
    }
    let pair_p =
        (2.0 * crate::pt(q / std::f64::consts::SQRT_2, df, true, false) - 1.0).clamp(0.0, 1.0);
    let comparisons = rr.max(1.0) * cc * (cc - 1.0) / 2.0;
    tail_result(pair_p.powf(comparisons), lower_tail, log_p)
}

pub fn qtukey(p: f64, rr: f64, cc: f64, df: f64, lower_tail: bool, log_p: bool) -> f64 {
    if p.is_nan() || rr.is_nan() || cc.is_nan() || df.is_nan() {
        return p + rr + cc + df;
    }
    if rr < 1.0 || cc < 2.0 || df <= 0.0 || q_p01_check(p, log_p) {
        return f64::NAN;
    }
    match q_p01_boundaries(p, lower_tail, log_p, 0.0, f64::INFINITY) {
        Ok(Some(boundary)) => return boundary,
        Ok(None) => {}
        Err(value) => return value,
    }
    let target = probability_from_args(p, lower_tail, log_p);
    let mut hi = 1.0;
    while ptukey(hi, rr, cc, df, true, false) < target {
        hi *= 2.0;
        if !hi.is_finite() {
            return f64::INFINITY;
        }
    }
    bisect(0.0, hi, |x| ptukey(x, rr, cc, df, true, false), target)
}

fn poisson_mixture_prob<F>(theta: f64, component: F) -> f64
where
    F: FnMut(f64) -> f64,
{
    poisson_mixture_raw(theta, component).clamp(0.0, 1.0)
}

fn poisson_mixture_raw<F>(theta: f64, mut component: F) -> f64
where
    F: FnMut(f64) -> f64,
{
    if theta == 0.0 {
        return component(0.0);
    }
    if theta > 700.0 {
        let k = theta.floor();
        return component(k);
    }
    let mut weight = (-theta).exp();
    let mut sum = weight * component(0.0);
    let mut total_w = weight;
    for k in 1..100_000 {
        weight *= theta / k as f64;
        let add = weight * component(k as f64);
        sum += add;
        total_w += weight;
        if weight < POIS_EPS * total_w && k as f64 > theta {
            break;
        }
    }
    sum / total_w
}

fn integrate_noncentral_t(x: f64, df: f64, ncp: f64) -> f64 {
    let upper = crate::qchisq(0.999_999, df, true, false)
        .max(df + 12.0 * (2.0 * df).sqrt())
        .max(1.0);
    let steps = 512usize;
    let h = upper / steps as f64;
    let mut sum = 0.0;
    for i in 0..=steps {
        let v = i as f64 * h;
        let weight = if i == 0 || i == steps {
            1.0
        } else if i % 2 == 0 {
            2.0
        } else {
            4.0
        };
        let arg = if v == 0.0 {
            if x >= 0.0 {
                f64::INFINITY
            } else {
                f64::NEG_INFINITY
            }
        } else {
            x * (v / df).sqrt() - ncp
        };
        sum += weight * crate::pnorm(arg, 0.0, 1.0, true, false) * crate::dchisq(v, df, false);
    }
    sum * h / 3.0
}

fn bisect<F>(mut lo: f64, mut hi: f64, cdf: F, target: f64) -> f64
where
    F: Fn(f64) -> f64,
{
    for _ in 0..160 {
        let mid = 0.5 * (lo + hi);
        if cdf(mid) < target {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    0.5 * (lo + hi)
}

fn tail_result(lower_prob: f64, lower_tail: bool, log_p: bool) -> f64 {
    let prob = if lower_tail {
        lower_prob
    } else {
        1.0 - lower_prob
    };
    d_val(prob.clamp(0.0, 1.0), log_p)
}

fn probability_from_args(p: f64, lower_tail: bool, log_p: bool) -> f64 {
    if log_p {
        if lower_tail {
            p.exp()
        } else {
            -p.exp_m1()
        }
    } else if lower_tail {
        p
    } else {
        1.0 - p
    }
}
