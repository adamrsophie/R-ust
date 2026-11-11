const M_LN2: f64 = std::f64::consts::LN_2;
const M_2PI: f64 = std::f64::consts::TAU;
const M_SQRT2: f64 = std::f64::consts::SQRT_2;
const M_SQRT_32: f64 = 5.656_854_249_492_380_195_206_754_896_838;
const M_1_SQRT_2PI: f64 = 0.398_942_280_401_432_677_939_946_059_934;
const M_LN_SQRT_2PI: f64 = 0.918_938_533_204_672_741_780_329_736_406;

fn r_d_0(log_p: bool) -> f64 {
    if log_p {
        f64::NEG_INFINITY
    } else {
        0.0
    }
}

fn r_d_1(log_p: bool) -> f64 {
    if log_p {
        0.0
    } else {
        1.0
    }
}

fn r_dt_0(lower_tail: bool, log_p: bool) -> f64 {
    if lower_tail {
        r_d_0(log_p)
    } else {
        r_d_1(log_p)
    }
}

fn r_dt_1(lower_tail: bool, log_p: bool) -> f64 {
    if lower_tail {
        r_d_1(log_p)
    } else {
        r_d_0(log_p)
    }
}

fn r_d_lval(p: f64, lower_tail: bool) -> f64 {
    if lower_tail {
        p
    } else {
        0.5 - p + 0.5
    }
}

fn r_d_cval(p: f64, lower_tail: bool) -> f64 {
    if lower_tail {
        0.5 - p + 0.5
    } else {
        p
    }
}

fn r_dt_qiv(p: f64, lower_tail: bool, log_p: bool) -> f64 {
    if log_p {
        if lower_tail {
            p.exp()
        } else {
            -p.exp_m1()
        }
    } else {
        r_d_lval(p, lower_tail)
    }
}

fn r_dt_civ(p: f64, lower_tail: bool, log_p: bool) -> f64 {
    if log_p {
        if lower_tail {
            -p.exp_m1()
        } else {
            p.exp()
        }
    } else {
        r_d_cval(p, lower_tail)
    }
}

fn force_int(x: f64) -> f64 {
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

pub fn dnorm(x: f64, mean: f64, sd: f64, give_log: bool) -> f64 {
    if x.is_nan() || mean.is_nan() || sd.is_nan() {
        return x + mean + sd;
    }
    if sd < 0.0 {
        return f64::NAN;
    }
    if !sd.is_finite() {
        return r_d_0(give_log);
    }
    if !x.is_finite() && mean == x {
        return f64::NAN;
    }
    if sd == 0.0 {
        return if x == mean {
            f64::INFINITY
        } else {
            r_d_0(give_log)
        };
    }

    let x = (x - mean) / sd;
    if !x.is_finite() {
        return r_d_0(give_log);
    }

    let x = x.abs();
    if x >= 2.0 * f64::MAX.sqrt() {
        return r_d_0(give_log);
    }
    if give_log {
        return -(M_LN_SQRT_2PI + 0.5 * x * x + sd.ln());
    }
    if x < 5.0 {
        return M_1_SQRT_2PI * (-0.5 * x * x).exp() / sd;
    }

    if x > (-2.0 * M_LN2 * (f64::MIN_EXP as f64 + 1.0 - f64::MANTISSA_DIGITS as f64)).sqrt() {
        return 0.0;
    }

    let x1 = (force_int(x * 65_536.0)) / 65_536.0;
    let x2 = x - x1;
    M_1_SQRT_2PI / sd * ((-0.5 * x1 * x1).exp() * ((-0.5 * x2 - x1) * x2).exp())
}

pub fn pnorm(q: f64, mean: f64, sd: f64, lower_tail: bool, log_p: bool) -> f64 {
    if q.is_nan() || mean.is_nan() || sd.is_nan() {
        return q + mean + sd;
    }
    if !q.is_finite() && mean == q {
        return f64::NAN;
    }
    if sd <= 0.0 {
        if sd < 0.0 {
            return f64::NAN;
        }
        return if q < mean {
            r_dt_0(lower_tail, log_p)
        } else {
            r_dt_1(lower_tail, log_p)
        };
    }
    let x = (q - mean) / sd;
    if !x.is_finite() {
        return if q < mean {
            r_dt_0(lower_tail, log_p)
        } else {
            r_dt_1(lower_tail, log_p)
        };
    }

    let (p, cp) = pnorm_both(x, if lower_tail { 0 } else { 1 }, log_p);
    if lower_tail {
        p
    } else {
        cp
    }
}

pub fn pnorm_both(x: f64, i_tail: i32, log_p: bool) -> (f64, f64) {
    const A: [f64; 5] = [
        2.235_252_035_460_683_928_7,
        161.028_231_068_555_878_81,
        1067.689_485_460_370_958_2,
        18154.981_253_343_561_249,
        0.065_682_337_918_207_449_113,
    ];
    const B: [f64; 4] = [
        47.202_581_904_688_241_87,
        976.098_551_737_776_693_22,
        10260.932_208_618_978_205,
        45507.789_335_026_729_956,
    ];
    const C: [f64; 9] = [
        0.398_941_512_088_134_667_64,
        8.883_149_794_388_375_941_2,
        93.506_656_132_177_855_979,
        597.270_276_394_800_262_26,
        2494.537_585_290_372_671_1,
        6848.190_450_536_282_332_6,
        11602.651_437_647_350_124,
        9842.714_838_383_978_021_8,
        1.076_557_677_372_019_231_7e-8,
    ];
    const D: [f64; 8] = [
        22.266_688_044_328_115_691,
        235.387_901_782_624_998_61,
        1519.377_599_407_554_805,
        6485.558_298_266_760_755,
        18615.571_640_885_098_091,
        34900.952_721_145_977_266,
        38912.003_286_093_271_411,
        19685.429_676_859_990_727,
    ];
    const P: [f64; 6] = [
        0.215_898_534_057_956_99,
        0.127_401_161_160_247_363_9,
        0.022_235_277_870_649_807,
        0.001_421_619_193_227_893_466,
        2.911_287_495_116_879_2e-5,
        0.023_073_441_764_940_173_03,
    ];
    const Q: [f64; 5] = [
        1.284_260_096_144_911_21,
        0.468_238_212_480_865_118,
        0.065_988_137_868_928_551_5,
        0.003_782_396_332_027_582_44,
        7.297_515_550_839_662_05e-5,
    ];

    if x.is_nan() {
        return (x, x);
    }

    let lower = i_tail != 1;
    let upper = i_tail != 0;
    let y = x.abs();
    let eps = f64::EPSILON * 0.5;
    let mut cum = 0.0;
    let mut ccum = 0.0;

    if y <= 0.674_489_75 {
        let (xnum, xden) = if y > eps {
            let xsq = x * x;
            let mut xnum = A[4] * xsq;
            let mut xden = xsq;
            for i in 0..3 {
                xnum = (xnum + A[i]) * xsq;
                xden = (xden + B[i]) * xsq;
            }
            (xnum, xden)
        } else {
            (0.0, 0.0)
        };
        let temp = x * (xnum + A[3]) / (xden + B[3]);
        if lower {
            cum = 0.5 + temp;
        }
        if upper {
            ccum = 0.5 - temp;
        }
        if log_p {
            if lower {
                cum = cum.ln();
            }
            if upper {
                ccum = ccum.ln();
            }
        }
    } else if y <= M_SQRT_32 {
        let mut xnum = C[8] * y;
        let mut xden = y;
        for i in 0..7 {
            xnum = (xnum + C[i]) * y;
            xden = (xden + D[i]) * y;
        }
        let temp = (xnum + C[7]) / (xden + D[7]);
        let (mut c, mut cc) = pnorm_do_del(y, x, temp, log_p, lower, upper);
        if x > 0.0 {
            std::mem::swap(&mut c, &mut cc);
        }
        cum = c;
        ccum = cc;
    } else if (log_p && y < 1e170)
        || (lower && -38.4674 < x && x < 8.2924)
        || (upper && -8.2924 < x && x < 38.4674)
    {
        let xsq = 1.0 / (x * x);
        let mut xnum = P[5] * xsq;
        let mut xden = xsq;
        for i in 0..4 {
            xnum = (xnum + P[i]) * xsq;
            xden = (xden + Q[i]) * xsq;
        }
        let temp = xsq * (xnum + P[4]) / (xden + Q[4]);
        let temp = (M_1_SQRT_2PI - temp) / y;
        let (mut c, mut cc) = pnorm_do_del(x, x, temp, log_p, lower, upper);
        if x > 0.0 {
            std::mem::swap(&mut c, &mut cc);
        }
        cum = c;
        ccum = cc;
    } else if x > 0.0 {
        cum = r_d_1(log_p);
        ccum = r_d_0(log_p);
    } else {
        cum = r_d_0(log_p);
        ccum = r_d_1(log_p);
    }

    (cum, ccum)
}

fn pnorm_do_del(
    x_for_split: f64,
    original_x: f64,
    temp: f64,
    log_p: bool,
    lower: bool,
    upper: bool,
) -> (f64, f64) {
    let xsq = (x_for_split * 16.0).trunc() / 16.0;
    let del = (x_for_split - xsq) * (x_for_split + xsq);
    if log_p {
        let cum = (-xsq * (xsq * 0.5)) - (del * 0.5) + temp.ln();
        let ccum = if (lower && original_x > 0.0) || (upper && original_x <= 0.0) {
            (-(-xsq * (xsq * 0.5)).exp() * (-(del * 0.5)).exp() * temp).ln_1p()
        } else {
            0.0
        };
        (cum, ccum)
    } else {
        let cum = (-xsq * (xsq * 0.5)).exp() * (-(del * 0.5)).exp() * temp;
        let ccum = 1.0 - cum;
        (cum, ccum)
    }
}

pub fn qnorm(p: f64, mean: f64, sd: f64, lower_tail: bool, log_p: bool) -> f64 {
    if p.is_nan() || mean.is_nan() || sd.is_nan() {
        return p + mean + sd;
    }

    if log_p {
        if p > 0.0 {
            return f64::NAN;
        }
        if p == 0.0 {
            return if lower_tail {
                f64::INFINITY
            } else {
                f64::NEG_INFINITY
            };
        }
        if p == f64::NEG_INFINITY {
            return if lower_tail {
                f64::NEG_INFINITY
            } else {
                f64::INFINITY
            };
        }
    } else {
        if !(0.0..=1.0).contains(&p) {
            return f64::NAN;
        }
        if p == 0.0 {
            return if lower_tail {
                f64::NEG_INFINITY
            } else {
                f64::INFINITY
            };
        }
        if p == 1.0 {
            return if lower_tail {
                f64::INFINITY
            } else {
                f64::NEG_INFINITY
            };
        }
    }

    if sd < 0.0 {
        return f64::NAN;
    }
    if sd == 0.0 {
        return mean;
    }

    let p_ = r_dt_qiv(p, lower_tail, log_p);
    let q = p_ - 0.5;

    let mut val;
    if q.abs() <= 0.425 {
        let r = 0.180625 - q * q;
        val = q
            * (((((((r * 2509.080_928_730_122_672_7 + 33430.575_583_588_128_105) * r
                + 67265.770_927_008_700_853)
                * r
                + 45921.953_931_549_871_457)
                * r
                + 13731.693_765_509_461_125)
                * r
                + 1971.590_950_306_551_442_7)
                * r
                + 133.141_667_891_784_377_45)
                * r
                + 3.387_132_872_796_366_608)
            / (((((((r * 5226.495_278_852_854_561 + 28729.085_735_721_942_674) * r
                + 39307.895_800_092_710_61)
                * r
                + 21213.794_301_586_595_867)
                * r
                + 5394.196_021_424_751_107_7)
                * r
                + 687.187_007_492_057_908_3)
                * r
                + 42.313_330_701_600_911_252)
                * r
                + 1.0);
    } else {
        let lp = if log_p && ((lower_tail && q <= 0.0) || (!lower_tail && q > 0.0)) {
            p
        } else {
            (if q > 0.0 {
                r_dt_civ(p, lower_tail, log_p)
            } else {
                p_
            })
            .ln()
        };
        let mut r = (-lp).sqrt();
        if r <= 5.0 {
            r += -1.6;
            val = (((((((r * 7.745_450_142_783_414_076_4e-4
                + 0.022_723_844_989_269_184_583_3)
                * r
                + 0.241_780_725_177_450_611_77)
                * r
                + 1.270_458_252_452_368_382_58)
                * r
                + 3.647_848_324_763_204_605_04)
                * r
                + 5.769_497_221_460_691_405_5)
                * r
                + 4.630_337_846_156_545_295_9)
                * r
                + 1.423_437_110_749_683_577_34)
                / (((((((r * 1.050_750_071_644_416_843_24e-9 + 5.475_938_084_995_344_946e-4)
                    * r
                    + 0.015_198_666_563_616_457_196_6)
                    * r
                    + 0.148_103_976_427_480_074_59)
                    * r
                    + 0.689_767_334_985_100_004_55)
                    * r
                    + 1.676_384_830_183_803_849_4)
                    * r
                    + 2.053_191_626_637_758_821_87)
                    * r
                    + 1.0);
        } else if r <= 27.0 {
            r += -5.0;
            val = (((((((r * 2.010_334_399_292_288_132_65e-7
                + 2.711_555_568_743_487_578_15e-5)
                * r
                + 0.001_242_660_947_388_078_438_6)
                * r
                + 0.026_532_189_526_576_123_093)
                * r
                + 0.296_560_571_828_504_891_23)
                * r
                + 1.784_826_539_917_291_335_8)
                * r
                + 5.463_784_911_164_114_369_9)
                * r
                + 6.657_904_643_501_103_777_2)
                / (((((((r * 2.044_263_103_389_939_785_64e-15
                    + 1.421_511_758_316_445_888_7e-7)
                    * r
                    + 1.846_318_317_510_054_681_8e-5)
                    * r
                    + 7.868_691_311_456_132_591e-4)
                    * r
                    + 0.014_875_361_290_850_614_852_5)
                    * r
                    + 0.136_929_880_922_735_805_31)
                    * r
                    + 0.599_832_206_555_887_937_69)
                    * r
                    + 1.0);
        } else if r >= 6.4e8 {
            val = r * M_SQRT2;
        } else {
            let s2 = -2.0 * lp;
            let mut x2 = s2 - (M_2PI * s2).ln();
            if r < 36_000.0 {
                x2 = s2 - (M_2PI * x2).ln() - 2.0 / (2.0 + x2);
                if r < 840.0 {
                    x2 = s2 - (M_2PI * x2).ln()
                        + 2.0 * (-(1.0 - 1.0 / (4.0 + x2)) / (2.0 + x2)).ln_1p();
                    if r < 109.0 {
                        x2 = s2 - (M_2PI * x2).ln()
                            + 2.0
                                * (-(1.0 - (1.0 - 5.0 / (6.0 + x2)) / (4.0 + x2)) / (2.0 + x2))
                                    .ln_1p();
                        if r < 55.0 {
                            x2 = s2 - (M_2PI * x2).ln()
                                + 2.0
                                    * (-(1.0
                                        - (1.0 - (5.0 - 9.0 / (8.0 + x2)) / (6.0 + x2))
                                            / (4.0 + x2))
                                        / (2.0 + x2))
                                        .ln_1p();
                        }
                    }
                }
            }
            val = x2.sqrt();
        }
        if q < 0.0 {
            val = -val;
        }
    }

    mean + sd * val
}
