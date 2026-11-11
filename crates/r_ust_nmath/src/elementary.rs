const M_LN2: f64 = std::f64::consts::LN_2;

pub fn R_finite(x: f64) -> i32 {
    x.is_finite() as i32
}

pub fn R_isnancpp(x: f64) -> i32 {
    x.is_nan() as i32
}

pub fn fmax2(x: f64, y: f64) -> f64 {
    if x.is_nan() || y.is_nan() {
        x + y
    } else if x < y {
        y
    } else {
        x
    }
}

pub fn fmin2(x: f64, y: f64) -> f64 {
    if x.is_nan() || y.is_nan() {
        x + y
    } else if x < y {
        x
    } else {
        y
    }
}

pub const fn imax2(x: i32, y: i32) -> i32 {
    if x < y {
        y
    } else {
        x
    }
}

pub const fn imin2(x: i32, y: i32) -> i32 {
    if x < y {
        x
    } else {
        y
    }
}

pub fn sign(x: f64) -> f64 {
    if x.is_nan() {
        x
    } else if x > 0.0 {
        1.0
    } else if x == 0.0 {
        0.0
    } else {
        -1.0
    }
}

pub fn fsign(x: f64, y: f64) -> f64 {
    if x.is_nan() || y.is_nan() {
        x + y
    } else if y >= 0.0 {
        x.abs()
    } else {
        -x.abs()
    }
}

pub fn ftrunc(x: f64) -> f64 {
    x.trunc()
}

pub fn cospi(mut x: f64) -> f64 {
    if x.is_nan() {
        return x;
    }
    if !x.is_finite() {
        return f64::NAN;
    }
    x = x.abs() % 2.0;
    if x % 1.0 == 0.5 {
        0.0
    } else if x == 1.0 {
        -1.0
    } else if x == 0.0 {
        1.0
    } else {
        (std::f64::consts::PI * x).cos()
    }
}

pub fn sinpi(mut x: f64) -> f64 {
    if x.is_nan() {
        return x;
    }
    if !x.is_finite() {
        return f64::NAN;
    }
    x %= 2.0;
    if x <= -1.0 {
        x += 2.0;
    } else if x > 1.0 {
        x -= 2.0;
    }
    if x == 0.0 || x == 1.0 {
        0.0
    } else if x == 0.5 {
        1.0
    } else if x == -0.5 {
        -1.0
    } else {
        (std::f64::consts::PI * x).sin()
    }
}

pub fn Rtanpi(mut x: f64) -> f64 {
    if x.is_nan() {
        return x;
    }
    if !x.is_finite() {
        return f64::NAN;
    }
    x %= 1.0;
    if x <= -0.5 {
        x += 1.0;
    } else if x > 0.5 {
        x -= 1.0;
    }
    if x == 0.0 {
        0.0
    } else if x == 0.5 {
        f64::NAN
    } else if x == 0.25 {
        1.0
    } else if x == -0.25 {
        -1.0
    } else {
        (std::f64::consts::PI * x).tan()
    }
}

pub fn tanpi(x: f64) -> f64 {
    Rtanpi(x)
}

pub fn log1pexp(x: f64) -> f64 {
    if x <= 18.0 {
        x.exp().ln_1p()
    } else if x > 33.3 {
        x
    } else {
        x + (-x).exp()
    }
}

pub fn log1mexp(x: f64) -> f64 {
    log1_exp(-x)
}

pub fn log1pmx(x: f64) -> f64 {
    const MIN_LOG1_VALUE: f64 = -0.791_490_64;

    if !(MIN_LOG1_VALUE..=1.0).contains(&x) {
        x.ln_1p() - x
    } else {
        let r = x / (2.0 + x);
        let y = r * r;
        if x.abs() < 1e-2 {
            let two = 2.0;
            r * ((((two / 9.0 * y + two / 7.0) * y + two / 5.0) * y + two / 3.0) * y - x)
        } else {
            r * (2.0 * y * logcf(y, 3.0, 2.0, 1e-14) - x)
        }
    }
}

pub fn logspace_add(logx: f64, logy: f64) -> f64 {
    fmax2(logx, logy) + (-(logx - logy).abs()).exp().ln_1p()
}

pub fn logspace_sub(logx: f64, logy: f64) -> f64 {
    logx + log1_exp(logy - logx)
}

pub fn logspace_sum(logx: &[f64]) -> f64 {
    match logx.len() {
        0 => f64::NEG_INFINITY,
        1 => logx[0],
        2 => logspace_add(logx[0], logx[1]),
        _ => {
            let max = logx
                .iter()
                .copied()
                .fold(logx[0], |acc, value| if acc < value { value } else { acc });
            let sum: f64 = logx.iter().map(|value| (*value - max).exp()).sum();
            max + sum.ln()
        }
    }
}

pub fn pow1p(x: f64, y: f64) -> f64 {
    if y.is_nan() {
        return if x == 0.0 { 1.0 } else { y };
    }
    if 0.0 <= y && y == y.trunc() && y <= 4.0 {
        match y as i32 {
            0 => return 1.0,
            1 => return x + 1.0,
            2 => return x * (x + 2.0) + 1.0,
            3 => return x * (x * (x + 3.0) + 3.0) + 1.0,
            4 => return x * (x * (x * (x + 4.0) + 6.0) + 4.0) + 1.0,
            _ => {}
        }
    }
    let xp1 = x + 1.0;
    let x_roundtrip = xp1 - 1.0;
    if x_roundtrip == x || x.abs() > 0.5 || x.is_nan() {
        xp1.powf(y)
    } else {
        (y * x.ln_1p()).exp()
    }
}

pub(crate) fn d_0(log_p: bool) -> f64 {
    if log_p {
        f64::NEG_INFINITY
    } else {
        0.0
    }
}

pub(crate) fn d_1(log_p: bool) -> f64 {
    if log_p {
        0.0
    } else {
        1.0
    }
}

pub(crate) fn dt_0(lower_tail: bool, log_p: bool) -> f64 {
    if lower_tail {
        d_0(log_p)
    } else {
        d_1(log_p)
    }
}

pub(crate) fn dt_1(lower_tail: bool, log_p: bool) -> f64 {
    if lower_tail {
        d_1(log_p)
    } else {
        d_0(log_p)
    }
}

pub(crate) fn d_val(x: f64, log_p: bool) -> f64 {
    if log_p {
        x.ln()
    } else {
        x
    }
}

pub(crate) fn d_exp(x: f64, log_p: bool) -> f64 {
    if log_p {
        x
    } else {
        x.exp()
    }
}

pub(crate) fn d_clog(p: f64, log_p: bool) -> f64 {
    if log_p {
        (-p).ln_1p()
    } else {
        0.5 - p + 0.5
    }
}

pub(crate) fn dt_qiv(p: f64, lower_tail: bool, log_p: bool) -> f64 {
    if log_p {
        if lower_tail {
            p.exp()
        } else {
            -p.exp_m1()
        }
    } else if lower_tail {
        p
    } else {
        0.5 - p + 0.5
    }
}

pub(crate) fn dt_clog(p: f64, lower_tail: bool, log_p: bool) -> f64 {
    if lower_tail {
        if log_p {
            log1_exp(p)
        } else {
            (-p).ln_1p()
        }
    } else if log_p {
        p
    } else {
        p.ln()
    }
}

pub(crate) fn force_int(x: f64) -> f64 {
    if !x.is_finite() || x == 0.0 {
        return x;
    }
    let floor = x.floor();
    let frac = x - floor;
    if frac < 0.5 {
        floor
    } else if frac > 0.5 {
        floor + 1.0
    } else if (floor / 2.0).fract() == 0.0 {
        floor
    } else {
        floor + 1.0
    }
}

pub(crate) fn nonint(x: f64) -> bool {
    (x - force_int(x)).abs() > 1e-7 * x.abs().max(1.0)
}

pub(crate) fn q_p01_check(p: f64, log_p: bool) -> bool {
    (log_p && p > 0.0) || (!log_p && !(0.0..=1.0).contains(&p))
}

pub(crate) fn q_p01_boundaries(
    p: f64,
    lower_tail: bool,
    log_p: bool,
    left: f64,
    right: f64,
) -> Result<Option<f64>, f64> {
    if log_p {
        if p > 0.0 {
            return Err(f64::NAN);
        }
        if p == 0.0 {
            return Ok(Some(if lower_tail { right } else { left }));
        }
        if p == f64::NEG_INFINITY {
            return Ok(Some(if lower_tail { left } else { right }));
        }
    } else {
        if !(0.0..=1.0).contains(&p) {
            return Err(f64::NAN);
        }
        if p == 0.0 {
            return Ok(Some(if lower_tail { left } else { right }));
        }
        if p == 1.0 {
            return Ok(Some(if lower_tail { right } else { left }));
        }
    }
    Ok(None)
}

pub(crate) fn log1_exp(x: f64) -> f64 {
    if x > -M_LN2 {
        (-(x.exp_m1())).ln()
    } else {
        (-x.exp()).ln_1p()
    }
}

pub(crate) fn logcf(x: f64, i: f64, d: f64, eps: f64) -> f64 {
    const SCALE_FACTOR: f64 = 1.157_920_892_373_162e77;

    let mut c1 = 2.0 * d;
    let mut c2 = i + d;
    let mut c4 = c2 + d;
    let mut a1 = c2;
    let mut b1 = i * (c2 - i * x);
    let b2 = d * d * x;
    let mut a2 = c4 * c2 - b2;
    let mut b2 = c4 * b1 - i * b2;

    while (a2 * b1 - a1 * b2).abs() > (eps * b1 * b2).abs() {
        let mut c3 = c2 * c2 * x;
        c2 += d;
        c4 += d;
        a1 = c4 * a2 - c3 * a1;
        b1 = c4 * b2 - c3 * b1;

        c3 = c1 * c1 * x;
        c1 += d;
        c4 += d;
        a2 = c4 * a1 - c3 * a2;
        b2 = c4 * b1 - c3 * b2;

        if b2.abs() > SCALE_FACTOR {
            a1 /= SCALE_FACTOR;
            b1 /= SCALE_FACTOR;
            a2 /= SCALE_FACTOR;
            b2 /= SCALE_FACTOR;
        } else if b2.abs() < 1.0 / SCALE_FACTOR {
            a1 *= SCALE_FACTOR;
            b1 *= SCALE_FACTOR;
            a2 *= SCALE_FACTOR;
            b2 *= SCALE_FACTOR;
        }
    }

    a2 / b2
}
