pub fn fround(x: f64, digits: f64) -> f64 {
    if x.is_nan() || digits.is_nan() {
        return x + digits;
    }
    if !x.is_finite() || x == 0.0 || digits == f64::INFINITY {
        return x;
    }
    if digits == f64::NEG_INFINITY {
        return 0.0_f64.copysign(x);
    }
    let digits = digits.clamp(-308.0, 308.0).trunc();
    let pow = 10.0_f64.powf(digits.abs());
    if !pow.is_finite() {
        return x;
    }
    if digits >= 0.0 {
        round_ties_even(x * pow) / pow
    } else {
        round_ties_even(x / pow) * pow
    }
}

pub fn fprec(x: f64, digits: f64) -> f64 {
    if x.is_nan() || digits.is_nan() {
        return x + digits;
    }
    if !x.is_finite() || x == 0.0 || digits == f64::INFINITY {
        return x;
    }
    if digits <= 0.0 || digits == f64::NEG_INFINITY {
        return 0.0_f64.copysign(x);
    }
    let digits = digits.clamp(1.0, 308.0).trunc();
    let mag = x.abs().log10().floor() + 1.0;
    fround(x, digits - mag)
}

fn round_ties_even(x: f64) -> f64 {
    if !x.is_finite() {
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
