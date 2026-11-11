use crate::elementary::{
    d_0, d_exp, d_val, dt_0, dt_1, force_int, nonint, q_p01_boundaries, q_p01_check,
};

pub fn dsignrank(x: f64, n: f64, give_log: bool) -> f64 {
    if x.is_nan() || n.is_nan() {
        return x + n;
    }
    if n < 0.0 || nonint(n) {
        return f64::NAN;
    }
    if nonint(x) || x < 0.0 {
        return d_0(give_log);
    }
    let counts = signrank_counts(force_int(n) as usize);
    let x = force_int(x) as usize;
    if x >= counts.len() {
        return d_0(give_log);
    }
    d_exp(counts[x].ln() - (n * std::f64::consts::LN_2), give_log)
}

pub fn psignrank(x: f64, n: f64, lower_tail: bool, log_p: bool) -> f64 {
    if x.is_nan() || n.is_nan() {
        return x + n;
    }
    if n < 0.0 || nonint(n) {
        return f64::NAN;
    }
    let counts = signrank_counts(force_int(n) as usize);
    if x < 0.0 {
        return dt_0(lower_tail, log_p);
    }
    if x >= counts.len() as f64 - 1.0 {
        return dt_1(lower_tail, log_p);
    }
    let upto = force_int(x.floor()) as usize;
    let total: f64 = counts.iter().sum();
    let prob = counts.iter().take(upto + 1).sum::<f64>() / total;
    tail_result(prob, lower_tail, log_p)
}

pub fn qsignrank(p: f64, n: f64, lower_tail: bool, log_p: bool) -> f64 {
    if p.is_nan() || n.is_nan() {
        return p + n;
    }
    if n < 0.0 || nonint(n) || q_p01_check(p, log_p) {
        return f64::NAN;
    }
    let counts = signrank_counts(force_int(n) as usize);
    let hi = counts.len().saturating_sub(1) as f64;
    match q_p01_boundaries(p, lower_tail, log_p, 0.0, hi) {
        Ok(Some(boundary)) => return boundary,
        Ok(None) => {}
        Err(value) => return value,
    }
    discrete_quantile(&counts, probability_from_args(p, lower_tail, log_p))
}

pub fn rsignrank_from_state(state: &mut crate::RngState, n: f64) -> f64 {
    if n < 0.0 || nonint(n) {
        return f64::NAN;
    }
    let mut out = 0.0;
    for i in 1..=force_int(n) as usize {
        if state.unif_rand() < 0.5 {
            out += i as f64;
        }
    }
    out
}

pub fn signrank_free() {}

pub fn dwilcox(x: f64, m: f64, n: f64, give_log: bool) -> f64 {
    if x.is_nan() || m.is_nan() || n.is_nan() {
        return x + m + n;
    }
    if invalid_wilcox(m, n) {
        return f64::NAN;
    }
    if nonint(x) || x < 0.0 {
        return d_0(give_log);
    }
    let counts = wilcox_counts(force_int(m) as usize, force_int(n) as usize);
    let x = force_int(x) as usize;
    if x >= counts.len() {
        return d_0(give_log);
    }
    let total: f64 = counts.iter().sum();
    d_exp(counts[x].ln() - total.ln(), give_log)
}

pub fn pwilcox(x: f64, m: f64, n: f64, lower_tail: bool, log_p: bool) -> f64 {
    if x.is_nan() || m.is_nan() || n.is_nan() {
        return x + m + n;
    }
    if invalid_wilcox(m, n) {
        return f64::NAN;
    }
    let counts = wilcox_counts(force_int(m) as usize, force_int(n) as usize);
    if x < 0.0 {
        return dt_0(lower_tail, log_p);
    }
    if x >= counts.len() as f64 - 1.0 {
        return dt_1(lower_tail, log_p);
    }
    let upto = force_int(x.floor()) as usize;
    let total: f64 = counts.iter().sum();
    let prob = counts.iter().take(upto + 1).sum::<f64>() / total;
    tail_result(prob, lower_tail, log_p)
}

pub fn qwilcox(p: f64, m: f64, n: f64, lower_tail: bool, log_p: bool) -> f64 {
    if p.is_nan() || m.is_nan() || n.is_nan() {
        return p + m + n;
    }
    if invalid_wilcox(m, n) || q_p01_check(p, log_p) {
        return f64::NAN;
    }
    let counts = wilcox_counts(force_int(m) as usize, force_int(n) as usize);
    let hi = counts.len().saturating_sub(1) as f64;
    match q_p01_boundaries(p, lower_tail, log_p, 0.0, hi) {
        Ok(Some(boundary)) => return boundary,
        Ok(None) => {}
        Err(value) => return value,
    }
    discrete_quantile(&counts, probability_from_args(p, lower_tail, log_p))
}

pub fn rwilcox_from_state(state: &mut crate::RngState, m: f64, n: f64) -> f64 {
    if invalid_wilcox(m, n) {
        return f64::NAN;
    }
    let m = force_int(m) as usize;
    let n = force_int(n) as usize;
    let total = m + n;
    let mut ranks: Vec<usize> = (1..=total).collect();
    for i in 0..m {
        let j = i + state.unif_index((total - i) as f64) as usize;
        ranks.swap(i, j);
    }
    let sum: usize = ranks.iter().take(m).sum();
    (sum - m * (m + 1) / 2) as f64
}

pub fn wilcox_free() {}

fn signrank_counts(n: usize) -> Vec<f64> {
    let max = n * (n + 1) / 2;
    let mut counts = vec![0.0; max + 1];
    counts[0] = 1.0;
    for rank in 1..=n {
        for sum in (rank..=max).rev() {
            counts[sum] += counts[sum - rank];
        }
    }
    counts
}

fn wilcox_counts(m: usize, n: usize) -> Vec<f64> {
    let total_ranks = m + n;
    let min_sum = m * (m + 1) / 2;
    let max_stat = m * n;
    let max_sum = min_sum + max_stat;
    let mut dp = vec![vec![0.0; max_sum + 1]; m + 1];
    dp[0][0] = 1.0;
    for rank in 1..=total_ranks {
        let max_k = m.min(rank);
        for k in (1..=max_k).rev() {
            for sum in (rank..=max_sum).rev() {
                dp[k][sum] += dp[k - 1][sum - rank];
            }
        }
    }
    let mut counts = vec![0.0; max_stat + 1];
    for (stat, count) in counts.iter_mut().enumerate() {
        *count = dp[m][stat + min_sum];
    }
    counts
}

fn invalid_wilcox(m: f64, n: f64) -> bool {
    m < 0.0 || n < 0.0 || nonint(m) || nonint(n) || !m.is_finite() || !n.is_finite()
}

fn discrete_quantile(counts: &[f64], target: f64) -> f64 {
    let total: f64 = counts.iter().sum();
    let mut sum = 0.0;
    for (i, count) in counts.iter().enumerate() {
        sum += *count / total;
        if sum >= target {
            return i as f64;
        }
    }
    counts.len().saturating_sub(1) as f64
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
