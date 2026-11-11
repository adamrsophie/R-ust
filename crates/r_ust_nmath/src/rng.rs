#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RngState {
    i1: u32,
    i2: u32,
}

impl Default for RngState {
    fn default() -> Self {
        Self::new(1234, 5678)
    }
}

impl RngState {
    pub const fn new(i1: u32, i2: u32) -> Self {
        Self { i1, i2 }
    }

    pub const fn seed(&self) -> (u32, u32) {
        (self.i1, self.i2)
    }

    pub fn set_seed(&mut self, i1: u32, i2: u32) {
        self.i1 = i1;
        self.i2 = i2;
    }

    pub fn unif_rand(&mut self) -> f64 {
        self.i1 = 36969u32
            .wrapping_mul(self.i1 & 0o177777)
            .wrapping_add(self.i1 >> 16);
        self.i2 = 18000u32
            .wrapping_mul(self.i2 & 0o177777)
            .wrapping_add(self.i2 >> 16);
        ((self.i1 << 16) ^ (self.i2 & 0o177777)) as f64 * 2.328_306_437_080_797e-10
    }

    pub fn unif_index(&mut self, dn: f64) -> f64 {
        if dn <= 0.0 {
            return 0.0;
        }
        let bits = dn.log2().ceil() as i32;
        loop {
            let value = self.rbits(bits);
            if value < dn {
                return value;
            }
        }
    }

    pub fn norm_rand_inversion(&mut self) -> f64 {
        const BIG: f64 = 134_217_728.0;
        let u1 = (BIG * self.unif_rand()) as i32 as f64 + self.unif_rand();
        crate::qnorm(u1 / BIG, 0.0, 1.0, true, false)
    }

    pub fn exp_rand(&mut self) -> f64 {
        const Q: [f64; 16] = [
            std::f64::consts::LN_2,
            0.933_373_687_519_045_9,
            0.988_877_796_183_867_5,
            0.998_495_925_291_496_0,
            0.999_829_281_106_138_9,
            0.999_983_316_410_072_7,
            0.999_998_569_143_876_7,
            0.999_999_890_692_555_8,
            0.999_999_992_473_415_9,
            0.999_999_999_528_327_5,
            0.999_999_999_972_881_4,
            0.999_999_999_998_559_8,
            0.999_999_999_999_928_9,
            0.999_999_999_999_996_8,
            0.999_999_999_999_999_9,
            1.000_000_000_000_000_0,
        ];

        let mut a = 0.0;
        let mut u = self.unif_rand();
        while u <= 0.0 || u >= 1.0 {
            u = self.unif_rand();
        }
        loop {
            u += u;
            if u > 1.0 {
                break;
            }
            a += Q[0];
        }
        u -= 1.0;
        if u <= Q[0] {
            return a + u;
        }

        let mut i = 0;
        let mut ustar = self.unif_rand();
        let mut umin = ustar;
        loop {
            ustar = self.unif_rand();
            if umin > ustar {
                umin = ustar;
            }
            i += 1;
            if u <= Q[i] {
                break;
            }
        }
        a + umin * Q[0]
    }

    pub fn runif(&mut self, min: f64, max: f64) -> f64 {
        if !min.is_finite() || !max.is_finite() || max < min {
            return f64::NAN;
        }
        if min == max {
            return min;
        }
        loop {
            let u = self.unif_rand();
            if u > 0.0 && u < 1.0 {
                return min + (max - min) * u;
            }
        }
    }

    pub fn rnorm_inversion(&mut self, mean: f64, sd: f64) -> f64 {
        if mean.is_nan() || sd.is_nan() || sd < 0.0 {
            return f64::NAN;
        }
        if sd == 0.0 || !sd.is_finite() {
            return mean;
        }
        mean + sd * self.norm_rand_inversion()
    }

    pub fn rexp(&mut self, scale: f64) -> f64 {
        if !scale.is_finite() || scale <= 0.0 {
            return if scale == 0.0 { 0.0 } else { f64::NAN };
        }
        scale * self.exp_rand()
    }

    pub fn rcauchy(&mut self, location: f64, scale: f64) -> f64 {
        if location.is_nan() || !scale.is_finite() || scale < 0.0 {
            return f64::NAN;
        }
        if scale == 0.0 || !location.is_finite() {
            return location;
        }
        location + scale * (std::f64::consts::PI * self.unif_rand()).tan()
    }

    pub fn rlogis(&mut self, location: f64, scale: f64) -> f64 {
        if location.is_nan() || !scale.is_finite() {
            return f64::NAN;
        }
        if scale == 0.0 || !location.is_finite() {
            return location;
        }
        let u = self.unif_rand();
        location + scale * (u / (1.0 - u)).ln()
    }

    pub fn rweibull(&mut self, shape: f64, scale: f64) -> f64 {
        if !shape.is_finite() || !scale.is_finite() || shape <= 0.0 || scale <= 0.0 {
            return if scale == 0.0 { 0.0 } else { f64::NAN };
        }
        scale * (-self.unif_rand().ln()).powf(1.0 / shape)
    }

    pub fn rlnorm(&mut self, meanlog: f64, sdlog: f64) -> f64 {
        if meanlog.is_nan() || !sdlog.is_finite() || sdlog < 0.0 {
            return f64::NAN;
        }
        self.rnorm_inversion(meanlog, sdlog).exp()
    }

    pub fn rpois(&mut self, lambda: f64) -> f64 {
        crate::discrete::rpois_from_state(self, lambda)
    }

    pub fn rgeom(&mut self, p: f64) -> f64 {
        crate::discrete::rgeom_from_state(self, p)
    }

    pub fn rhyper(&mut self, r: f64, b: f64, n: f64) -> f64 {
        crate::hypergeometric::rhyper_from_state(self, r, b, n)
    }

    pub fn rbinom(&mut self, n: f64, p: f64) -> f64 {
        crate::discrete::rbinom_from_state(self, n, p)
    }

    pub fn rgamma(&mut self, shape: f64, scale: f64) -> f64 {
        if !shape.is_finite() || !scale.is_finite() || shape < 0.0 || scale <= 0.0 {
            return f64::NAN;
        }
        if shape == 0.0 {
            return 0.0;
        }
        if shape < 1.0 {
            return self.rgamma(shape + 1.0, scale) * self.unif_rand().powf(1.0 / shape);
        }
        let d = shape - 1.0 / 3.0;
        let c = 1.0 / (9.0 * d).sqrt();
        loop {
            let x = self.norm_rand_inversion();
            let v = 1.0 + c * x;
            if v <= 0.0 {
                continue;
            }
            let v = v * v * v;
            let u = self.unif_rand();
            if u < 1.0 - 0.0331 * x.powi(4) {
                return scale * d * v;
            }
            if u.ln() < 0.5 * x * x + d * (1.0 - v + v.ln()) {
                return scale * d * v;
            }
        }
    }

    pub fn rchisq(&mut self, df: f64) -> f64 {
        self.rgamma(df / 2.0, 2.0)
    }

    pub fn rbeta(&mut self, a: f64, b: f64) -> f64 {
        if a <= 0.0 || b <= 0.0 || !a.is_finite() || !b.is_finite() {
            return f64::NAN;
        }
        let x = self.rgamma(a, 1.0);
        let y = self.rgamma(b, 1.0);
        x / (x + y)
    }

    pub fn rf(&mut self, m: f64, n: f64) -> f64 {
        if m <= 0.0 || n <= 0.0 || m.is_nan() || n.is_nan() {
            return f64::NAN;
        }
        (self.rchisq(m) / m) / (self.rchisq(n) / n)
    }

    pub fn rt(&mut self, df: f64) -> f64 {
        if df <= 0.0 || df.is_nan() {
            return f64::NAN;
        }
        self.norm_rand_inversion() / (self.rchisq(df) / df).sqrt()
    }

    pub fn rnbinom(&mut self, size: f64, prob: f64) -> f64 {
        if size < 0.0 || prob <= 0.0 || prob > 1.0 || !size.is_finite() || !prob.is_finite() {
            return f64::NAN;
        }
        if size == 0.0 {
            return 0.0;
        }
        let lambda = self.rgamma(size, (1.0 - prob) / prob);
        self.rpois(lambda)
    }

    pub fn rnbinom_mu(&mut self, size: f64, mu: f64) -> f64 {
        if size < 0.0 || mu < 0.0 || !size.is_finite() || !mu.is_finite() {
            return f64::NAN;
        }
        if size == 0.0 {
            return 0.0;
        }
        let lambda = self.rgamma(size, mu / size);
        self.rpois(lambda)
    }

    pub fn rnchisq(&mut self, df: f64, ncp: f64) -> f64 {
        if df < 0.0 || ncp < 0.0 || !df.is_finite() || !ncp.is_finite() {
            return f64::NAN;
        }
        if ncp == 0.0 {
            return self.rchisq(df);
        }
        let extra = self.rpois(ncp / 2.0);
        self.rchisq(df + 2.0 * extra)
    }

    pub fn rsignrank(&mut self, n: f64) -> f64 {
        crate::rank::rsignrank_from_state(self, n)
    }

    pub fn rwilcox(&mut self, m: f64, n: f64) -> f64 {
        crate::rank::rwilcox_from_state(self, m, n)
    }

    pub fn rmultinom(&mut self, n: i32, prob: &[f64], out: &mut [i32]) {
        crate::hypergeometric::rmultinom_from_state(self, n, prob, out)
    }

    fn rbits(&mut self, bits: i32) -> f64 {
        let mut v: i64 = 0;
        let mut n = 0;
        while n <= bits {
            let v1 = (self.unif_rand() * 65_536.0).floor() as i64;
            v = 65_536 * v + v1;
            n += 16;
        }
        (v & ((1_i64 << bits) - 1)) as f64
    }
}
