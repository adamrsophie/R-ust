use crate::elementary::{
    d_0, d_1, d_exp, d_val, dt_0, dt_1, force_int, nonint, q_p01_boundaries, q_p01_check,
};
use crate::special::lchoose;

pub fn dhyper(x: f64, r: f64, b: f64, n: f64, give_log: bool) -> f64 {
    if x.is_nan() || r.is_nan() || b.is_nan() || n.is_nan() {
        return x + r + b + n;
    }
    if invalid_params(r, b, n) {
        return f64::NAN;
    }
    if nonint(x) {
        return d_0(give_log);
    }
    let x = force_int(x);
    let lo = 0.0_f64.max(n - b);
    let hi = r.min(n);
    if x < lo || x > hi {
        return d_0(give_log);
    }
    let logv = lchoose(r, x) + lchoose(b, n - x) - lchoose(r + b, n);
    d_exp(logv, give_log)
}

pub fn phyper(x: f64, r: f64, b: f64, n: f64, lower_tail: bool, log_p: bool) -> f64 {
    if x.is_nan() || r.is_nan() || b.is_nan() || n.is_nan() {
        return x + r + b + n;
    }
    if invalid_params(r, b, n) {
        return f64::NAN;
    }
    let lo = 0.0_f64.max(n - b);
    let hi = r.min(n);
    if x < lo {
        return dt_0(lower_tail, log_p);
    }
    if x >= hi {
        return dt_1(lower_tail, log_p);
    }
    let upto = force_int(x.floor()).min(hi);
    let mut sum = 0.0;
    let mut k = lo;
    while k <= upto {
        sum += dhyper(k, r, b, n, false);
        k += 1.0;
    }
    tail_result(sum.clamp(0.0, 1.0), lower_tail, log_p)
}

pub fn qhyper(p: f64, r: f64, b: f64, n: f64, lower_tail: bool, log_p: bool) -> f64 {
    if p.is_nan() || r.is_nan() || b.is_nan() || n.is_nan() {
        return p + r + b + n;
    }
    if invalid_params(r, b, n) || q_p01_check(p, log_p) {
        return f64::NAN;
    }
    let lo = 0.0_f64.max(n - b);
    let hi = r.min(n);
    match q_p01_boundaries(p, lower_tail, log_p, lo, hi) {
        Ok(Some(boundary)) => return boundary,
        Ok(None) => {}
        Err(value) => return value,
    }
    let target = probability_from_args(p, lower_tail, log_p);
    let mut sum = 0.0;
    let mut k = lo;
    while k <= hi {
        sum += dhyper(k, r, b, n, false);
        if sum >= target {
            return k;
        }
        k += 1.0;
    }
    hi
}

pub fn rhyper_from_state(state: &mut crate::RngState, r: f64, b: f64, n: f64) -> f64 {
    if invalid_params(r, b, n) {
        return f64::NAN;
    }
    let mut red = force_int(r);
    let mut black = force_int(b);
    let mut draws = force_int(n);
    let mut count = 0.0;
    while draws > 0.0 {
        let total = red + black;
        if total <= 0.0 {
            break;
        }
        if state.unif_rand() < red / total {
            count += 1.0;
            red -= 1.0;
        } else {
            black -= 1.0;
        }
        draws -= 1.0;
    }
    count
}

pub fn rmultinom_from_state(state: &mut crate::RngState, n: i32, prob: &[f64], out: &mut [i32]) {
    if n < 0 || prob.len() != out.len() || prob.is_empty() {
        for value in out {
            *value = -1;
        }
        return;
    }
    out.fill(0);
    let mut remaining_n = n;
    let mut remaining_p: f64 = prob.iter().sum();
    if !remaining_p.is_finite()
        || remaining_p <= 0.0
        || prob.iter().any(|p| !p.is_finite() || *p < 0.0)
    {
        for value in out {
            *value = -1;
        }
        return;
    }
    for i in 0..prob.len().saturating_sub(1) {
        if remaining_n <= 0 {
            break;
        }
        let p = if remaining_p > 0.0 {
            prob[i] / remaining_p
        } else {
            0.0
        };
        let draw = state.rbinom(remaining_n as f64, p).min(remaining_n as f64) as i32;
        out[i] = draw;
        remaining_n -= draw;
        remaining_p -= prob[i];
    }
    if let Some(last) = out.last_mut() {
        *last = remaining_n;
    }
}

fn invalid_params(r: f64, b: f64, n: f64) -> bool {
    !r.is_finite()
        || !b.is_finite()
        || !n.is_finite()
        || r < 0.0
        || b < 0.0
        || n < 0.0
        || nonint(r)
        || nonint(b)
        || nonint(n)
        || n > r + b
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

#[allow(dead_code)]
fn _one(log_p: bool) -> f64 {
    d_1(log_p)
}
