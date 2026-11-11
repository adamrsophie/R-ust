use crate::elementary::{force_int, nonint};

const EULER: f64 = 0.577_215_664_901_532_9;

pub fn digamma(x: f64) -> f64 {
    if x.is_nan() {
        return x;
    }
    if x == f64::NEG_INFINITY || (x <= 0.0 && x == x.trunc()) {
        return f64::NAN;
    }
    if x == f64::INFINITY {
        return x;
    }
    if x < 0.0 {
        return digamma(1.0 - x) - std::f64::consts::PI / (std::f64::consts::PI * x).tan();
    }
    if x == 0.0 {
        return f64::NEG_INFINITY;
    }
    if x == 1.0 {
        return -EULER;
    }

    let mut x = x;
    let mut acc = 0.0;
    while x < 8.0 {
        acc -= 1.0 / x;
        x += 1.0;
    }
    let inv = 1.0 / x;
    let inv2 = inv * inv;
    acc + x.ln()
        - 0.5 * inv
        - inv2
            * (1.0 / 12.0
                - inv2
                    * (1.0 / 120.0
                        - inv2 * (1.0 / 252.0 - inv2 * (1.0 / 240.0 - inv2 * 5.0 / 660.0))))
}

pub fn trigamma(x: f64) -> f64 {
    psigamma(x, 1.0)
}

pub fn tetragamma(x: f64) -> f64 {
    psigamma(x, 2.0)
}

pub fn pentagamma(x: f64) -> f64 {
    psigamma(x, 3.0)
}

pub fn psigamma(x: f64, deriv: f64) -> f64 {
    if x.is_nan() || deriv.is_nan() {
        return x + deriv;
    }
    if deriv < 0.0 || nonint(deriv) {
        return f64::NAN;
    }
    let n = force_int(deriv) as i32;
    if n == 0 {
        return digamma(x);
    }
    if x == 1.0 {
        match n {
            1 => return std::f64::consts::PI.powi(2) / 6.0,
            2 => return -2.404_113_806_319_188_5,
            3 => return std::f64::consts::PI.powi(4) / 15.0,
            _ => {}
        }
    }
    if x <= 0.0 {
        return f64::NAN;
    }
    let n_usize = n as usize;
    let fact = factorial(n_usize);
    let sign = if n % 2 == 0 { -1.0 } else { 1.0 };
    sign * fact * hurwitz_zeta_positive((n + 1) as usize, x)
}

pub fn dpsifn(x: f64, n: i32, kode: i32, m: i32) -> (Vec<f64>, i32, i32) {
    if n < 0 || m < 0 || x.is_nan() {
        return (vec![f64::NAN; m.max(0) as usize], 0, 1);
    }
    let mut ans = Vec::with_capacity(m as usize);
    for j in 0..m {
        let order = n + j;
        let mut value = psigamma(x, order as f64);
        if order == 0 {
            value = -value;
            if kode == 2 {
                value += x.ln();
            }
        } else {
            value *= if order % 2 == 0 { -1.0 } else { 1.0 } / factorial(order as usize);
        }
        ans.push(value);
    }
    (ans, 0, 0)
}

fn hurwitz_zeta_positive(power: usize, x: f64) -> f64 {
    let mut x = x;
    let mut sum = 0.0;
    while x < 12.0 {
        sum += x.powi(-(power as i32));
        x += 1.0;
    }

    let s = power as f64;
    let inv = 1.0 / x;
    let inv_s = x.powf(-s);
    sum += x.powf(1.0 - s) / (s - 1.0) + 0.5 * inv_s;

    let bernoulli = [
        1.0 / 6.0,
        -1.0 / 30.0,
        1.0 / 42.0,
        -1.0 / 30.0,
        5.0 / 66.0,
        -691.0 / 2730.0,
    ];
    let mut rising = s;
    let mut inv2_pow = inv * inv;
    for (k, b) in bernoulli.iter().enumerate() {
        if k > 0 {
            rising *= s + (2 * k - 1) as f64;
            rising *= s + (2 * k) as f64;
        }
        let denom = factorial(2 * (k + 1)) / factorial(2 * k);
        let term = *b * rising / denom * inv2_pow * inv_s;
        sum += term;
        inv2_pow *= inv * inv;
        if term.abs() < sum.abs() * 1e-15 {
            break;
        }
    }
    sum
}

fn factorial(n: usize) -> f64 {
    (2..=n).fold(1.0, |acc, value| acc * value as f64)
}

#[allow(dead_code)]
fn digamma_at_one() -> f64 {
    -EULER
}
