#![allow(non_snake_case)]
// Allow long float literals so numerical constants keep their full published precision.
#![allow(clippy::excessive_precision, clippy::inconsistent_digit_grouping)]

mod bessel;
mod cdf_quantiles;
mod continuous_densities;
mod discrete;
mod elementary;
mod hypergeometric;
mod noncentral;
mod normal;
mod polygamma;
mod rank;
mod rng;
mod rounding;
mod simple_distributions;
mod special;
mod surface;

pub use bessel::{
    bessel_i, bessel_i_ex, bessel_j, bessel_j_ex, bessel_k, bessel_k_ex, bessel_y, bessel_y_ex,
};
pub use cdf_quantiles::{
    pbeta, pchisq, pf, pgamma, pnbinom, pnbinom_mu, pt, qbeta, qchisq, qf, qgamma, qnbinom,
    qnbinom_mu, qt,
};
pub use continuous_densities::{dbeta, dchisq, df, dgamma, dnbinom, dnbinom_mu, dnchisq, dt};
pub use discrete::{
    dbinom, dbinom_raw, dgeom, dpois, dpois_raw, pbinom, pgeom, ppois, qbinom, qgeom, qpois,
};
pub use elementary::{
    cospi, fmax2, fmin2, fsign, ftrunc, imax2, imin2, log1mexp, log1pexp, log1pmx, logspace_add,
    logspace_sub, logspace_sum, pow1p, sign, sinpi, tanpi, R_finite, R_isnancpp, Rtanpi,
};
pub use hypergeometric::{dhyper, phyper, qhyper};
pub use noncentral::{
    dnbeta, dnf, dnt, pnbeta, pnchisq, pnf, pnt, ptukey, qnbeta, qnchisq, qnf, qnt, qtukey,
};
pub use normal::{dnorm, pnorm, pnorm_both, qnorm};
pub use polygamma::{digamma, dpsifn, pentagamma, psigamma, tetragamma, trigamma};
pub use rank::{
    dsignrank, dwilcox, psignrank, pwilcox, qsignrank, qwilcox, signrank_free, wilcox_free,
};
pub use rng::RngState;
pub use rounding::{fprec, fround};
pub use simple_distributions::{
    dcauchy, dexp, dlnorm, dlogis, dunif, dweibull, pcauchy, pexp, plnorm, plogis, punif, pweibull,
    qcauchy, qexp, qlnorm, qlogis, qunif, qweibull,
};
pub use special::{beta, choose, gammafn, lbeta, lchoose, lgamma1p, lgammafn, lgammafn_sign};
pub use surface::{IMPLEMENTED_FUNCTIONS, PUBLIC_FUNCTIONS, UNIMPLEMENTED_FUNCTIONS};

use std::sync::Mutex;

static GLOBAL_RNG: Mutex<RngState> = Mutex::new(RngState::new(1234, 5678));

pub fn Rlog1p(x: f64) -> f64 {
    x.ln_1p()
}

pub fn R_pow(x: f64, y: f64) -> f64 {
    if x == 1.0 || y == 0.0 {
        return 1.0;
    }
    if x == 0.0 {
        if y > 0.0 {
            return 0.0;
        }
        if y < 0.0 {
            return f64::INFINITY;
        }
        return y;
    }
    if x.is_finite() && y.is_finite() {
        return x.powf(y);
    }
    if x.is_nan() || y.is_nan() {
        return x + y;
    }
    if !x.is_finite() {
        if x > 0.0 {
            return if y < 0.0 { 0.0 } else { f64::INFINITY };
        }
        if y.is_finite() && y == y.floor() {
            return if y < 0.0 {
                0.0
            } else if y_mod_2(y) != 0.0 {
                x
            } else {
                -x
            };
        }
    }
    if !y.is_finite() && x >= 0.0 {
        if y > 0.0 {
            return if x >= 1.0 { f64::INFINITY } else { 0.0 };
        }
        return if x < 1.0 { f64::INFINITY } else { 0.0 };
    }
    f64::NAN
}

pub fn R_pow_di(mut x: f64, n: i32) -> f64 {
    if x.is_nan() {
        return x;
    }
    let mut pow = 1.0;
    let mut n = i64::from(n);
    if n != 0 {
        if !x.is_finite() {
            return R_pow(x, n as f64);
        }
        if n < 0 {
            n = -n;
            x = 1.0 / x;
        }
        loop {
            if n & 1 != 0 {
                pow *= x;
            }
            n >>= 1;
            if n == 0 {
                break;
            }
            x *= x;
        }
    }
    pow
}

pub fn set_seed(i1: u32, i2: u32) {
    GLOBAL_RNG.lock().unwrap().set_seed(i1, i2);
}

pub fn get_seed() -> (u32, u32) {
    GLOBAL_RNG.lock().unwrap().seed()
}

pub fn unif_rand() -> f64 {
    GLOBAL_RNG.lock().unwrap().unif_rand()
}

pub fn R_unif_index(dn: f64) -> f64 {
    GLOBAL_RNG.lock().unwrap().unif_index(dn)
}

pub fn norm_rand() -> f64 {
    GLOBAL_RNG.lock().unwrap().norm_rand_inversion()
}

pub fn exp_rand() -> f64 {
    GLOBAL_RNG.lock().unwrap().exp_rand()
}

pub fn runif(min: f64, max: f64) -> f64 {
    GLOBAL_RNG.lock().unwrap().runif(min, max)
}

pub fn rnorm(mean: f64, sd: f64) -> f64 {
    GLOBAL_RNG.lock().unwrap().rnorm_inversion(mean, sd)
}

pub fn rexp(scale: f64) -> f64 {
    GLOBAL_RNG.lock().unwrap().rexp(scale)
}

pub fn rcauchy(location: f64, scale: f64) -> f64 {
    GLOBAL_RNG.lock().unwrap().rcauchy(location, scale)
}

pub fn rlogis(location: f64, scale: f64) -> f64 {
    GLOBAL_RNG.lock().unwrap().rlogis(location, scale)
}

pub fn rweibull(shape: f64, scale: f64) -> f64 {
    GLOBAL_RNG.lock().unwrap().rweibull(shape, scale)
}

pub fn rlnorm(meanlog: f64, sdlog: f64) -> f64 {
    GLOBAL_RNG.lock().unwrap().rlnorm(meanlog, sdlog)
}

pub fn rpois(lambda: f64) -> f64 {
    GLOBAL_RNG.lock().unwrap().rpois(lambda)
}

pub fn rgeom(p: f64) -> f64 {
    GLOBAL_RNG.lock().unwrap().rgeom(p)
}

pub fn rhyper(r: f64, b: f64, n: f64) -> f64 {
    GLOBAL_RNG.lock().unwrap().rhyper(r, b, n)
}

pub fn rbinom(n: f64, p: f64) -> f64 {
    GLOBAL_RNG.lock().unwrap().rbinom(n, p)
}

pub fn rgamma(shape: f64, scale: f64) -> f64 {
    GLOBAL_RNG.lock().unwrap().rgamma(shape, scale)
}

pub fn rchisq(df: f64) -> f64 {
    GLOBAL_RNG.lock().unwrap().rchisq(df)
}

pub fn rbeta(a: f64, b: f64) -> f64 {
    GLOBAL_RNG.lock().unwrap().rbeta(a, b)
}

pub fn rf(m: f64, n: f64) -> f64 {
    GLOBAL_RNG.lock().unwrap().rf(m, n)
}

pub fn rt(df: f64) -> f64 {
    GLOBAL_RNG.lock().unwrap().rt(df)
}

pub fn rnbinom(size: f64, prob: f64) -> f64 {
    GLOBAL_RNG.lock().unwrap().rnbinom(size, prob)
}

pub fn rnbinom_mu(size: f64, mu: f64) -> f64 {
    GLOBAL_RNG.lock().unwrap().rnbinom_mu(size, mu)
}

pub fn rnchisq(df: f64, ncp: f64) -> f64 {
    GLOBAL_RNG.lock().unwrap().rnchisq(df, ncp)
}

pub fn rsignrank(n: f64) -> f64 {
    GLOBAL_RNG.lock().unwrap().rsignrank(n)
}

pub fn rwilcox(m: f64, n: f64) -> f64 {
    GLOBAL_RNG.lock().unwrap().rwilcox(m, n)
}

pub fn rmultinom(n: i32, prob: &[f64], out: &mut [i32]) {
    GLOBAL_RNG.lock().unwrap().rmultinom(n, prob, out)
}

fn y_mod_2(y: f64) -> f64 {
    y - (y / 2.0).floor() * 2.0
}
