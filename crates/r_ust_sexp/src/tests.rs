use super::{is_na_real, SexpType, NA_REAL};

#[test]
fn missing_double_payload_matches_r() {
    assert!(NA_REAL.is_nan());
    assert!(is_na_real(NA_REAL));
    assert!(is_na_real(-NA_REAL));
    assert!(is_na_real(f64::from_bits(0x7ff8_0000_0000_07a2)));
    assert!(!is_na_real(f64::NAN));
    assert!(!is_na_real(f64::from_bits(0x0000_0000_0000_07a2)));
    assert!(!is_na_real(f64::from_bits(0x7ff8_0000_0000_07a3)));
    assert!(!is_na_real(1954.0));
    assert!(!is_na_real(f64::INFINITY));
}

#[test]
fn sexptype_values_match_r_4_6_0() {
    assert_eq!(SexpType::Nil.as_r_type(), 0);
    assert_eq!(SexpType::Closure.as_r_type(), 3);
    assert_eq!(SexpType::Integer.as_r_type(), 13);
    assert_eq!(SexpType::Raw.as_r_type(), 24);
    assert_eq!(SexpType::Object.as_r_type(), 25);
    assert_eq!(SexpType::Free.as_r_type(), 31);
    assert_eq!(SexpType::Function.as_r_type(), 99);
}
