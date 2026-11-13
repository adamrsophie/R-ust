use r_ust_parse::Parser;
use r_ust_sexp::Sexp;

#[test]
fn logical_aliases_are_symbols_while_reserved_logicals_are_values() {
    for (source, expected) in [
        ("T", Sexp::Symbol("T".into())),
        ("F", Sexp::Symbol("F".into())),
        ("`T`", Sexp::Symbol("T".into())),
        ("`F`", Sexp::Symbol("F".into())),
        ("TRUE", Sexp::Logical(vec![Some(true)])),
        ("FALSE", Sexp::Logical(vec![Some(false)])),
    ] {
        assert_eq!(
            Parser.parse_all(source).unwrap().expressions,
            vec![expected.clone()],
            "{source}"
        );
        let mut statements = Parser.parse_iter(source);
        assert_eq!(
            statements.next().unwrap().unwrap().expressions,
            vec![expected],
            "{source}"
        );
        assert!(statements.next().is_none());
    }
}
