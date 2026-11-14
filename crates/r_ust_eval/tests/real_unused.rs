use r_ust_eval::{EvalError, Frontend};

#[test]
fn unused_real_diagnostics_match_native_r_simple_deparse() {
    let fixture = include_str!("fixtures/real-unused-r-4.6.0.tsv");
    let mut rows = fixture.lines();
    assert_eq!(rows.next(), Some("source\terror_hex"));
    for row in rows {
        let (source, expected) = row.split_once('\t').unwrap();
        let mut frontend = Frontend::default();

        let result = frontend.eval_source(source);

        let Err(EvalError::Arity(message)) = result else {
            panic!("{source}: unexpected result {result:?}");
        };
        let encoded: String = message.bytes().map(|byte| format!("{byte:02x}")).collect();
        assert_eq!(encoded, expected, "{source}");
        assert_eq!(frontend.runtime_mut().take_output(), Vec::new(), "{source}");
    }
}
