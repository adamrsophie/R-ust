use r_ust_parse::Parser;
use r_ust_sexp::{
    canonical_language, AssignmentOperator, CallArgument, MemberName, Sexp, SexpType,
};

fn expression(source: &str) -> Sexp {
    Parser.parse_all(source).unwrap().expressions.remove(0)
}

fn call(name: &str, values: Vec<Sexp>) -> Sexp {
    Sexp::Call {
        function: Box::new(Sexp::Symbol(name.into())),
        arguments: values.into_iter().map(CallArgument::positional).collect(),
    }
}

#[test]
fn member_assignment_preserves_nested_target_operator_and_rhs() {
    for (operator, spelling) in [
        (AssignmentOperator::Left, "<-"),
        (AssignmentOperator::Equal, "="),
    ] {
        let parsed = expression(&format!("h[[1L]]$x {spelling} 9L"));
        let target = Sexp::Dollar {
            value: Box::new(Sexp::Extract {
                value: Box::new(Sexp::Symbol("h".into())),
                index: Box::new(Sexp::Integer(vec![Some(1)])),
            }),
            name: MemberName::Symbol("x".into()),
        };
        assert_eq!(
            parsed,
            Sexp::MemberAssignment {
                operator,
                target: Box::new(target),
                value: Box::new(Sexp::Integer(vec![Some(9)])),
            }
        );
        assert_eq!(parsed.sexptype(), SexpType::Language);
        assert_eq!(
            canonical_language(&parsed).unwrap(),
            call(
                spelling,
                vec![
                    call(
                        "$",
                        vec![
                            call(
                                "[[",
                                vec![Sexp::Symbol("h".into()), Sexp::Integer(vec![Some(1)])]
                            ),
                            Sexp::Symbol("x".into()),
                        ]
                    ),
                    Sexp::Integer(vec![Some(9)]),
                ]
            )
        );
    }
}

#[test]
fn simple_member_assignment_preserves_string_target_variants() {
    assert_eq!(
        expression("e$x<-9L"),
        Sexp::DollarAssignment {
            operator: AssignmentOperator::Left,
            target: "e".into(),
            name: MemberName::Symbol("x".into()),
            value: Box::new(Sexp::Integer(vec![Some(9)])),
        }
    );
    assert_eq!(
        expression("e[['x']]<-9L"),
        Sexp::ExtractAssignment {
            operator: AssignmentOperator::Left,
            target: "e".into(),
            index: Box::new(Sexp::String(vec![Some("x".into())])),
            value: Box::new(Sexp::Integer(vec![Some(9)])),
        }
    );
}

#[test]
fn member_assignment_rejects_non_symbol_roots_and_complex_superassignment() {
    for source in ["f()$x<-1L", "list(e)[[1L]]$x<-1L", "h[[1L]]$x<<-1L"] {
        assert!(Parser.parse_all(source).is_err(), "{source}");
    }
}
