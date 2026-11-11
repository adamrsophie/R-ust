use crate::elementary::{
    d_0, d_clog, d_exp, d_val, dt_0, dt_1, dt_clog, dt_qiv, log1_exp, log1pexp, q_p01_boundaries,
    q_p01_check, Rtanpi,
};

const M_1_SQRT_2PI: f64 = 0.398_942_280_401_432_677_939_946_059_934;
const M_LN_SQRT_2PI: f64 = 0.918_938_533_204_672_741_780_329_736_406;

pub fn dunif(x: f64, a: f64, b: f64, give_log: bool) -> f64 {
    if x.is_nan() || a.is_nan() || b.is_nan() {
        return x + a + b;
    }
    if b <= a {
        return f64::NAN;
    }
    if a <= x && x <= b {
        if give_log {
            -(b - a).ln()
        } else {
            1.0 / (b - a)
        }
    } else {
        d_0(give_log)
    }
}

pub fn punif(x: f64, a: f64, b: f64, lower_tail: bool, log_p: bool) -> f64 {
    if x.is_nan() || a.is_nan() || b.is_nan() {
        return x + a + b;
    }
    if b < a || !a.is_finite() || !b.is_finite() {
        return f64::NAN;
    }
    if x >= b {
        return dt_1(lower_tail, log_p);
    }
    if x <= a {
        return dt_0(lower_tail, log_p);
    }
    if lower_tail {
        d_val((x - a) / (b - a), log_p)
    } else {
        d_val((b - x) / (b - a), log_p)
    }
}

pub fn qunif(p: f64, a: f64, b: f64, lower_tail: bool, log_p: bool) -> f64 {
    if p.is_nan() || a.is_nan() || b.is_nan() {
        return p + a + b;
    }
    if q_p01_check(p, log_p) || !a.is_finite() || !b.is_finite() || b < a {
        return f64::NAN;
    }
    if b == a {
        return a;
    }
    a + dt_qiv(p, lower_tail, log_p) * (b - a)
}

pub fn dexp(x: f64, scale: f64, give_log: bool) -> f64 {
    if x.is_nan() || scale.is_nan() {
        return x + scale;
    }
    if scale <= 0.0 {
        return f64::NAN;
    }
    if x < 0.0 {
        d_0(give_log)
    } else if give_log {
        -x / scale - scale.ln()
    } else {
        (-x / scale).exp() / scale
    }
}

pub fn pexp(x: f64, scale: f64, lower_tail: bool, log_p: bool) -> f64 {
    if x.is_nan() || scale.is_nan() {
        return x + scale;
    }
    if scale < 0.0 {
        return f64::NAN;
    }
    if x <= 0.0 {
        return dt_0(lower_tail, log_p);
    }
    let x = -(x / scale);
    if lower_tail {
        if log_p {
            log1_exp(x)
        } else {
            -x.exp_m1()
        }
    } else {
        d_exp(x, log_p)
    }
}

pub fn qexp(p: f64, scale: f64, lower_tail: bool, log_p: bool) -> f64 {
    if p.is_nan() || scale.is_nan() {
        return p + scale;
    }
    if scale < 0.0 || q_p01_check(p, log_p) {
        return f64::NAN;
    }
    if p == dt_0(lower_tail, log_p) {
        return 0.0;
    }
    -scale * dt_clog(p, lower_tail, log_p)
}

pub fn dcauchy(x: f64, location: f64, scale: f64, give_log: bool) -> f64 {
    if x.is_nan() || location.is_nan() || scale.is_nan() {
        return x + location + scale;
    }
    if scale <= 0.0 {
        return f64::NAN;
    }
    let y = (x - location) / scale;
    if give_log {
        -(std::f64::consts::PI * scale * (1.0 + y * y)).ln()
    } else {
        1.0 / (std::f64::consts::PI * scale * (1.0 + y * y))
    }
}

pub fn pcauchy(mut x: f64, location: f64, scale: f64, lower_tail: bool, log_p: bool) -> f64 {
    if x.is_nan() || location.is_nan() || scale.is_nan() {
        return x + location + scale;
    }
    if scale <= 0.0 {
        return f64::NAN;
    }
    x = (x - location) / scale;
    if x.is_nan() {
        return f64::NAN;
    }
    if !x.is_finite() {
        return if x < 0.0 {
            dt_0(lower_tail, log_p)
        } else {
            dt_1(lower_tail, log_p)
        };
    }
    if !lower_tail {
        x = -x;
    }
    if x.abs() > 1.0 {
        let y = (1.0 / x).atan() / std::f64::consts::PI;
        if x > 0.0 {
            d_clog(y, log_p)
        } else {
            d_val(-y, log_p)
        }
    } else {
        d_val(0.5 + x.atan() / std::f64::consts::PI, log_p)
    }
}

pub fn qcauchy(mut p: f64, location: f64, scale: f64, mut lower_tail: bool, log_p: bool) -> f64 {
    if p.is_nan() || location.is_nan() || scale.is_nan() {
        return p + location + scale;
    }
    if q_p01_check(p, log_p) {
        return f64::NAN;
    }
    if scale <= 0.0 || !scale.is_finite() {
        return if scale == 0.0 { location } else { f64::NAN };
    }

    let my_inf = location + if lower_tail { scale } else { -scale } * f64::INFINITY;
    if log_p {
        if p > -1.0 {
            if p == 0.0 {
                return my_inf;
            }
            lower_tail = !lower_tail;
            p = -p.exp_m1();
        } else {
            p = p.exp();
        }
    } else if p > 0.5 {
        if p == 1.0 {
            return my_inf;
        }
        p = 1.0 - p;
        lower_tail = !lower_tail;
    }

    if p == 0.5 {
        location
    } else if p == 0.0 {
        location + if lower_tail { scale } else { -scale } * f64::NEG_INFINITY
    } else {
        location + if lower_tail { -scale } else { scale } / Rtanpi(p)
    }
}

pub fn dlogis(x: f64, location: f64, scale: f64, give_log: bool) -> f64 {
    if x.is_nan() || location.is_nan() || scale.is_nan() {
        return x + location + scale;
    }
    if scale <= 0.0 {
        return f64::NAN;
    }
    let x = ((x - location) / scale).abs();
    let e = (-x).exp();
    let f = 1.0 + e;
    if give_log {
        -(x + (scale * f * f).ln())
    } else {
        e / (scale * f * f)
    }
}

pub fn plogis(x: f64, location: f64, scale: f64, lower_tail: bool, log_p: bool) -> f64 {
    if x.is_nan() || location.is_nan() || scale.is_nan() {
        return x + location + scale;
    }
    if scale <= 0.0 {
        return f64::NAN;
    }
    let x = (x - location) / scale;
    if x.is_nan() {
        return f64::NAN;
    }
    if !x.is_finite() {
        return if x > 0.0 {
            dt_1(lower_tail, log_p)
        } else {
            dt_0(lower_tail, log_p)
        };
    }
    if log_p {
        -log1pexp(if lower_tail { -x } else { x })
    } else {
        1.0 / (1.0 + (if lower_tail { -x } else { x }).exp())
    }
}

pub fn qlogis(mut p: f64, location: f64, scale: f64, lower_tail: bool, log_p: bool) -> f64 {
    if p.is_nan() || location.is_nan() || scale.is_nan() {
        return p + location + scale;
    }
    match q_p01_boundaries(p, lower_tail, log_p, f64::NEG_INFINITY, f64::INFINITY) {
        Ok(Some(boundary)) => return boundary,
        Ok(None) => {}
        Err(value) => return value,
    }
    if scale < 0.0 {
        return f64::NAN;
    }
    if scale == 0.0 {
        return location;
    }
    if log_p {
        p = if lower_tail {
            p - log1_exp(p)
        } else {
            log1_exp(p) - p
        };
    } else {
        p = if lower_tail {
            (p / (1.0 - p)).ln()
        } else {
            ((1.0 - p) / p).ln()
        };
    }
    location + scale * p
}

pub fn dweibull(x: f64, shape: f64, scale: f64, give_log: bool) -> f64 {
    if x.is_nan() || shape.is_nan() || scale.is_nan() {
        return x + shape + scale;
    }
    if shape <= 0.0 || scale <= 0.0 {
        return f64::NAN;
    }
    if x < 0.0 || !x.is_finite() {
        return d_0(give_log);
    }
    if x == 0.0 && shape < 1.0 {
        return f64::INFINITY;
    }
    let tmp1 = (x / scale).powf(shape - 1.0);
    let tmp2 = tmp1 * (x / scale);
    if give_log {
        -tmp2 + (shape * tmp1 / scale).ln()
    } else {
        shape * tmp1 * (-tmp2).exp() / scale
    }
}

pub fn pweibull(x: f64, shape: f64, scale: f64, lower_tail: bool, log_p: bool) -> f64 {
    if x.is_nan() || shape.is_nan() || scale.is_nan() {
        return x + shape + scale;
    }
    if shape <= 0.0 || scale <= 0.0 {
        return f64::NAN;
    }
    if x <= 0.0 {
        return dt_0(lower_tail, log_p);
    }
    let x = -(x / scale).powf(shape);
    if lower_tail {
        if log_p {
            log1_exp(x)
        } else {
            -x.exp_m1()
        }
    } else {
        d_exp(x, log_p)
    }
}

pub fn qweibull(p: f64, shape: f64, scale: f64, lower_tail: bool, log_p: bool) -> f64 {
    if p.is_nan() || shape.is_nan() || scale.is_nan() {
        return p + shape + scale;
    }
    if shape <= 0.0 || scale <= 0.0 {
        return f64::NAN;
    }
    match q_p01_boundaries(p, lower_tail, log_p, 0.0, f64::INFINITY) {
        Ok(Some(boundary)) => return boundary,
        Ok(None) => {}
        Err(value) => return value,
    }
    scale * (-dt_clog(p, lower_tail, log_p)).powf(1.0 / shape)
}

pub fn dlnorm(x: f64, meanlog: f64, sdlog: f64, give_log: bool) -> f64 {
    if x.is_nan() || meanlog.is_nan() || sdlog.is_nan() {
        return x + meanlog + sdlog;
    }
    if sdlog < 0.0 {
        return f64::NAN;
    }
    if !x.is_finite() && x.ln() == meanlog {
        return f64::NAN;
    }
    if sdlog == 0.0 {
        return if x.ln() == meanlog {
            f64::INFINITY
        } else {
            d_0(give_log)
        };
    }
    if x <= 0.0 {
        return d_0(give_log);
    }
    let y = (x.ln() - meanlog) / sdlog;
    if give_log {
        -(M_LN_SQRT_2PI + 0.5 * y * y + (x * sdlog).ln())
    } else {
        M_1_SQRT_2PI * (-0.5 * y * y).exp() / (x * sdlog)
    }
}

pub fn plnorm(x: f64, meanlog: f64, sdlog: f64, lower_tail: bool, log_p: bool) -> f64 {
    if x.is_nan() || meanlog.is_nan() || sdlog.is_nan() {
        return x + meanlog + sdlog;
    }
    if sdlog < 0.0 {
        return f64::NAN;
    }
    if x > 0.0 {
        crate::pnorm(x.ln(), meanlog, sdlog, lower_tail, log_p)
    } else {
        dt_0(lower_tail, log_p)
    }
}

pub fn qlnorm(p: f64, meanlog: f64, sdlog: f64, lower_tail: bool, log_p: bool) -> f64 {
    if p.is_nan() || meanlog.is_nan() || sdlog.is_nan() {
        return p + meanlog + sdlog;
    }
    match q_p01_boundaries(p, lower_tail, log_p, 0.0, f64::INFINITY) {
        Ok(Some(boundary)) => return boundary,
        Ok(None) => {}
        Err(value) => return value,
    }
    crate::qnorm(p, meanlog, sdlog, lower_tail, log_p).exp()
}
