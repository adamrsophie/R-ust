use crate::elementary::{cospi, sinpi};
use crate::special::{gammafn, lgammafn};

const MAX_ITER: usize = 400;
const EPS: f64 = 1e-15;

pub fn bessel_j(x: f64, alpha: f64) -> f64 {
    bessel_j_ex(x, alpha, None)
}

pub fn bessel_j_ex(x: f64, alpha: f64, work: Option<&mut f64>) -> f64 {
    let value = bessel_j_value(x, alpha);
    if let Some(slot) = work {
        *slot = value;
    }
    value
}

pub fn bessel_i(x: f64, alpha: f64, expo: f64) -> f64 {
    bessel_i_ex(x, alpha, expo, None)
}

pub fn bessel_i_ex(x: f64, alpha: f64, expo: f64, work: Option<&mut f64>) -> f64 {
    let mut value = bessel_i_value(x, alpha);
    if expo != 0.0 && value.is_finite() {
        value *= (-x.abs()).exp();
    }
    if let Some(slot) = work {
        *slot = value;
    }
    value
}

pub fn bessel_y(x: f64, alpha: f64) -> f64 {
    bessel_y_ex(x, alpha, None)
}

pub fn bessel_y_ex(x: f64, alpha: f64, work: Option<&mut f64>) -> f64 {
    let value = bessel_y_value(x, alpha);
    if let Some(slot) = work {
        *slot = value;
    }
    value
}

pub fn bessel_k(x: f64, alpha: f64, expo: f64) -> f64 {
    bessel_k_ex(x, alpha, expo, None)
}

pub fn bessel_k_ex(x: f64, alpha: f64, expo: f64, work: Option<&mut f64>) -> f64 {
    let mut value = bessel_k_value(x, alpha);
    if expo != 0.0 && value.is_finite() {
        value *= x.exp();
    }
    if let Some(slot) = work {
        *slot = value;
    }
    value
}

fn bessel_j_value(x: f64, alpha: f64) -> f64 {
    if x.is_nan() || alpha.is_nan() {
        return x + alpha;
    }
    if x < 0.0 || !x.is_finite() || !alpha.is_finite() {
        return f64::NAN;
    }
    if x == 0.0 {
        return if alpha == 0.0 { 1.0 } else { 0.0 };
    }
    if alpha < 0.0 {
        let na = (-alpha).floor();
        if alpha + na == 0.0 {
            let sign = if (na as i64) % 2 == 0 { 1.0 } else { -1.0 };
            return sign * bessel_j_value(x, -alpha);
        }
        return bessel_j_value(x, -alpha) * cospi(alpha) - bessel_y_value(x, -alpha) * sinpi(alpha);
    }
    if x > 80.0 {
        return (2.0 / (std::f64::consts::PI * x)).sqrt()
            * (x - alpha * std::f64::consts::FRAC_PI_2 - std::f64::consts::FRAC_PI_4).cos();
    }
    series_j(x, alpha)
}

fn bessel_i_value(x: f64, alpha: f64) -> f64 {
    if x.is_nan() || alpha.is_nan() {
        return x + alpha;
    }
    if x < 0.0 || !x.is_finite() || !alpha.is_finite() {
        return f64::NAN;
    }
    if x == 0.0 {
        return if alpha == 0.0 { 1.0 } else { 0.0 };
    }
    if alpha < 0.0 {
        if alpha > -1.0 && sinpi(alpha).abs() > 1e-12 {
            return series_i(x, alpha);
        }
        let s = sinpi(-alpha);
        if s.abs() < 1e-12 {
            return bessel_i_value(x, -alpha);
        }
        return bessel_i_value(x, -alpha)
            + 2.0 / std::f64::consts::PI * s * bessel_k_value(x, -alpha);
    }
    if x > 80.0 {
        return x.exp() / (2.0 * std::f64::consts::PI * x).sqrt();
    }
    series_i(x, alpha)
}

fn bessel_y_value(x: f64, alpha: f64) -> f64 {
    if x.is_nan() || alpha.is_nan() {
        return x + alpha;
    }
    if x < 0.0 || !x.is_finite() || !alpha.is_finite() {
        return f64::NAN;
    }
    if x == 0.0 {
        return f64::NEG_INFINITY;
    }
    if x > 80.0 {
        return (2.0 / (std::f64::consts::PI * x)).sqrt()
            * (x - alpha * std::f64::consts::FRAC_PI_2 - std::f64::consts::FRAC_PI_4).sin();
    }
    let s = sinpi(alpha);
    if s.abs() < 1e-8 {
        let eps = 1e-7;
        return bessel_y_value(x, alpha + eps);
    }
    (bessel_j_value(x, alpha) * cospi(alpha) - series_j(x, -alpha)) / s
}

fn bessel_k_value(x: f64, alpha: f64) -> f64 {
    if x.is_nan() || alpha.is_nan() {
        return x + alpha;
    }
    if x < 0.0 || !x.is_finite() || !alpha.is_finite() {
        return f64::NAN;
    }
    if x == 0.0 {
        return f64::INFINITY;
    }
    let alpha = alpha.abs();
    if x > 80.0 {
        let corr = 1.0 + (4.0 * alpha * alpha - 1.0) / (8.0 * x);
        return (std::f64::consts::PI / (2.0 * x)).sqrt() * (-x).exp() * corr;
    }
    let s = sinpi(alpha);
    if s.abs() < 1e-8 {
        let eps = 1e-7;
        return bessel_k_value(x, alpha + eps);
    }
    std::f64::consts::PI * 0.5 * (series_i(x, -alpha) - bessel_i_value(x, alpha)) / s
}

fn series_j(x: f64, alpha: f64) -> f64 {
    let half = 0.5 * x;
    let mut term = (alpha * half.ln() - lgammafn(alpha + 1.0)).exp();
    if !term.is_finite() {
        return term;
    }
    let mut sum = term;
    let xsq4 = half * half;
    for k in 1..=MAX_ITER {
        term *= -xsq4 / (k as f64 * (alpha + k as f64));
        sum += term;
        if term.abs() <= EPS * sum.abs().max(1.0) {
            break;
        }
    }
    sum
}

fn series_i(x: f64, alpha: f64) -> f64 {
    let half = 0.5 * x;
    let mut term = (alpha * half.ln() - lgammafn(alpha + 1.0)).exp();
    if !term.is_finite() {
        return term;
    }
    let mut sum = term;
    let xsq4 = half * half;
    for k in 1..=MAX_ITER {
        term *= xsq4 / (k as f64 * (alpha + k as f64));
        sum += term;
        if term.abs() <= EPS * sum.abs().max(1.0) {
            break;
        }
    }
    sum
}

#[allow(dead_code)]
fn bessel_j_zero_order_at_zero() -> f64 {
    gammafn(1.0)
}
