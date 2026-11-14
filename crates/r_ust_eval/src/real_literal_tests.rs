use super::format_real;

#[test]
fn scalar_real_tokens_match_native_r_binary64_deparse() {
    let fixture = include_str!("../tests/fixtures/real-literal-r-4.6.0.tsv");
    let mut rows = fixture.lines();
    assert_eq!(
        rows.next(),
        Some("id\tbits\ttoken_hex\tsimpletoken_hex\tfullprint_hex")
    );
    for row in rows {
        let [id, bits, kept, simple, _]: [&str; 5] =
            row.split('\t').collect::<Vec<_>>().try_into().unwrap();
        let value = f64::from_bits(u64::from_str_radix(bits, 16).unwrap());
        for (keep_na, expected) in [(true, kept), (false, simple)] {
            let actual = format_real(value, keep_na);
            let actual_hex: String = actual.bytes().map(|byte| format!("{byte:02x}")).collect();
            assert_eq!(
                actual_hex, expected,
                "id={id} bits={bits} keep_na={keep_na}"
            );
        }
    }
}
