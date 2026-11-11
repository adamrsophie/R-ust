use crate::discrete::{dbinom_raw, dpois_raw};
use crate::elementary::{d_0, d_1, d_exp, d_val, force_int, nonint};
use crate::special::{bd0, lbeta, lgammafn, stirlerr};

const M_1_SQRT_2PI: f64 = 0.398_942_280_401_432_677_939_946_059_934;
const M_LN_SQRT_2PI: f64 = 0.918_938_533_204_672_741_780_329_736_406;

pub fn dbeta(x: f64, a: f64, b: f64, give_log: bool) -> f64 {
    if x.is_nan() || a.is_nan() || b.is_nan() {
        return x + a + b;
    }
    if a < 0.0 || b < 0.0 {
        return f64::NAN;
    }
    if !(0.0..=1.0).contains(&x) {
        return d_0(give_log);
    }
    if a == 0.0 || b == 0.0 || !a.is_finite() || !b.is_finite() {
        if a == 0.0 && b == 0.0 {
            return if x == 0.0 || x == 1.0 {
                f64::INFINITY
            } else {
                d_0(give_log)
            };
        }
        if a == 0.0 || a / b == 0.0 {
            return if x == 0.0 {
                f64::INFINITY
            } else {
                d_0(give_log)
            };
        }
        if b == 0.0 || b / a == 0.0 {
            return if x == 1.0 {
                f64::INFINITY
            } else {
                d_0(give_log)
            };
        }
        return if x == 0.5 {
            f64::INFINITY
        } else {
            d_0(give_log)
        };
    }
    if x == 0.0 {
        if a > 1.0 {
            return d_0(give_log);
        }
        if a < 1.0 {
            return f64::INFINITY;
        }
        return d_val(b, give_log);
    }
    if x == 1.0 {
        if b > 1.0 {
            return d_0(give_log);
        }
        if b < 1.0 {
            return f64::INFINITY;
        }
        return d_val(a, give_log);
    }
    let lval = if a <= 2.0 || b <= 2.0 {
        (a - 1.0) * x.ln() + (b - 1.0) * (-x).ln_1p() - lbeta(a, b)
    } else {
        (a + b - 1.0).ln() + dbinom_raw(a - 1.0, a + b - 2.0, x, 1.0 - x, true)
    };
    d_exp(lval, give_log)
}

pub fn dgamma(x: f64, shape: f64, scale: f64, give_log: bool) -> f64 {
    if x.is_nan() || shape.is_nan() || scale.is_nan() {
        return x + shape + scale;
    }
    if shape < 0.0 || scale <= 0.0 {
        return f64::NAN;
    }
    if x < 0.0 {
        return d_0(give_log);
    }
    if shape == 0.0 {
        return if x == 0.0 {
            f64::INFINITY
        } else {
            d_0(give_log)
        };
    }
    if x == 0.0 {
        if shape < 1.0 {
            return f64::INFINITY;
        }
        if shape > 1.0 {
            return d_0(give_log);
        }
        return if give_log { -scale.ln() } else { 1.0 / scale };
    }
    let pr;
    if shape < 1.0 {
        pr = dpois_raw(shape, x / scale, give_log);
        if give_log {
            pr + if (shape / x).is_finite() {
                (shape / x).ln()
            } else {
                shape.ln() - x.ln()
            }
        } else {
            pr * shape / x
        }
    } else {
        pr = dpois_raw(shape - 1.0, x / scale, give_log);
        if give_log {
            pr - scale.ln()
        } else {
            pr / scale
        }
    }
}

pub fn dchisq(x: f64, df: f64, give_log: bool) -> f64 {
    dgamma(x, df / 2.0, 2.0, give_log)
}

pub fn dt(x: f64, n: f64, give_log: bool) -> f64 {
    if x.is_nan() || n.is_nan() {
        return x + n;
    }
    if n <= 0.0 {
        return f64::NAN;
    }
    if !x.is_finite() {
        return d_0(give_log);
    }
    if !n.is_finite() {
        return crate::dnorm(x, 0.0, 1.0, give_log);
    }
    let t = -bd0(n / 2.0, (n + 1.0) / 2.0) + stirlerr((n + 1.0) / 2.0) - stirlerr(n / 2.0);
    let x2n = x * x / n;
    let (u, l_x2n, inv_sqrt) = if x2n > 1.0 / f64::EPSILON {
        let ax = x.abs();
        let l_x2n = ax.ln() - n.ln() / 2.0;
        (n * l_x2n, l_x2n, n.sqrt() / ax)
    } else if x2n > 0.2 {
        let l_x2n = (1.0 + x2n).ln() / 2.0;
        (n * l_x2n, l_x2n, (-l_x2n).exp())
    } else {
        let l_x2n = x2n.ln_1p() / 2.0;
        (
            -bd0(n / 2.0, (n + x * x) / 2.0) + x * x / 2.0,
            l_x2n,
            (-l_x2n).exp(),
        )
    };
    if give_log {
        t - u - (M_LN_SQRT_2PI + l_x2n)
    } else {
        (t - u).exp() * M_1_SQRT_2PI * inv_sqrt
    }
}

pub fn df(x: f64, m: f64, n: f64, give_log: bool) -> f64 {
    if x.is_nan() || m.is_nan() || n.is_nan() {
        return x + m + n;
    }
    if m <= 0.0 || n <= 0.0 {
        return f64::NAN;
    }
    if x < 0.0 {
        return d_0(give_log);
    }
    if x == 0.0 {
        return if m > 2.0 {
            d_0(give_log)
        } else if m == 2.0 {
            d_1(give_log)
        } else {
            f64::INFINITY
        };
    }
    if !m.is_finite() && !n.is_finite() {
        return if x == 1.0 {
            f64::INFINITY
        } else {
            d_0(give_log)
        };
    }
    if !n.is_finite() {
        return dgamma(x, m / 2.0, 2.0 / m, give_log);
    }
    if m > 1e14 {
        let dens = dgamma(1.0 / x, n / 2.0, 2.0 / n, give_log);
        return if give_log {
            dens - 2.0 * x.ln()
        } else {
            dens / (x * x)
        };
    }
    let f = 1.0 / (n + x * m);
    let q = n * f;
    let p = x * m * f;
    let (factor, dens) = if m >= 2.0 {
        (
            m * q / 2.0,
            dbinom_raw((m - 2.0) / 2.0, (m + n - 2.0) / 2.0, p, q, give_log),
        )
    } else {
        (
            m * m * q / (2.0 * p * (m + n)),
            dbinom_raw(m / 2.0, (m + n) / 2.0, p, q, give_log),
        )
    };
    if give_log {
        factor.ln() + dens
    } else {
        factor * dens
    }
}

pub fn dnbinom(x: f64, size: f64, prob: f64, give_log: bool) -> f64 {
    if x.is_nan() || size.is_nan() || prob.is_nan() {
        return x + size + prob;
    }
    if prob <= 0.0 || prob > 1.0 || size < 0.0 {
        return f64::NAN;
    }
    if nonint(x) || x < 0.0 || !x.is_finite() {
        return d_0(give_log);
    }
    let x = force_int(x);
    if x == 0.0 {
        if size == 0.0 {
            return d_1(give_log);
        }
        return if give_log {
            size * prob.ln()
        } else {
            prob.powf(size)
        };
    }
    let logv = lgammafn(x + size) - lgammafn(size) - lgammafn(x + 1.0)
        + size * prob.ln()
        + x * (-prob).ln_1p();
    d_exp(logv, give_log)
}

pub fn dnbinom_mu(x: f64, size: f64, mu: f64, give_log: bool) -> f64 {
    if x.is_nan() || size.is_nan() || mu.is_nan() {
        return x + size + mu;
    }
    if mu < 0.0 || size < 0.0 {
        return f64::NAN;
    }
    if nonint(x) || x < 0.0 || !x.is_finite() {
        return d_0(give_log);
    }
    if x == 0.0 && size == 0.0 {
        return d_1(give_log);
    }
    let x = force_int(x);
    if !size.is_finite() {
        return crate::dpois_raw(x, mu, give_log);
    }
    if x == 0.0 {
        return d_exp(
            size * if size < mu {
                (size / (size + mu)).ln()
            } else {
                (-mu / (size + mu)).ln_1p()
            },
            give_log,
        );
    }
    let prob = size / (size + mu);
    let logv = lgammafn(x + size) - lgammafn(size) - lgammafn(x + 1.0)
        + size * prob.ln()
        + x * (1.0 - prob).ln();
    d_exp(logv, give_log)
}

pub fn dnchisq(x: f64, df: f64, ncp: f64, give_log: bool) -> f64 {
    if x.is_nan() || df.is_nan() || ncp.is_nan() {
        return x + df + ncp;
    }
    if !df.is_finite() || !ncp.is_finite() || ncp < 0.0 || df < 0.0 {
        return f64::NAN;
    }
    if x < 0.0 || x == f64::INFINITY {
        return d_0(give_log);
    }
    if x == 0.0 && df < 2.0 {
        return f64::INFINITY;
    }
    if ncp == 0.0 {
        return if df > 0.0 {
            dchisq(x, df, give_log)
        } else {
            d_0(give_log)
        };
    }
    let ncp2 = 0.5 * ncp;
    let imax = ((-(2.0 + df) + ((2.0 - df) * (2.0 - df) + 4.0 * ncp * x).sqrt()) / 4.0)
        .ceil()
        .max(0.0);
    if !imax.is_finite() {
        return d_0(give_log);
    }
    let dfmid = df + 2.0 * imax;
    let mid = dpois_raw(imax, ncp2, false) * dchisq(x, dfmid, false);
    if mid == 0.0 {
        return d_0(give_log);
    }
    let mut sum = mid;
    let mut term = mid;
    let mut df_up = dfmid;
    let mut i = imax;
    let x2 = x * ncp2;
    loop {
        i += 1.0;
        let q = x2 / i / df_up;
        df_up += 2.0;
        term *= q;
        sum += term;
        if !(q >= 1.0 || term * q > (1.0 - q) * 5e-15 || term > 1e-10 * sum) {
            break;
        }
    }
    term = mid;
    let mut df_down = dfmid;
    i = imax;
    while i != 0.0 {
        df_down -= 2.0;
        let q = i * df_down / x2;
        i -= 1.0;
        term *= q;
        sum += term;
        if q < 1.0 && term * q <= (1.0 - q) * 5e-15 {
            break;
        }
    }
    d_val(sum, give_log)
}
