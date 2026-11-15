use r_ust_parse::Parser;
use r_ust_sexp::{CallArgument, Sexp};

#[test]
fn direct_string_call_heads_are_symbols_but_grouped_strings_stay_computed() {
    for (source, head) in [
        ("\"f\"(x)", Sexp::Symbol("f".into())),
        (
            "(\"f\")(x)",
            Sexp::Parenthesized(Box::new(Sexp::String(vec![Some("f".into())]))),
        ),
    ] {
        let actual = Parser.parse_all(source).unwrap().expressions;
        assert_eq!(
            actual,
            vec![Sexp::Call {
                function: Box::new(head),
                arguments: vec![CallArgument::positional(Sexp::Symbol("x".into()))],
            }]
        );
    }
}
