use r_ust_eval::{format_sexp, EvalError, Frontend, OutputEvent};
use r_ust_sexp::Sexp;

#[test]
fn primitive_handles_match_independent_r_values_visibility_and_effects() {
    for row in include_str!("fixtures/primitive-values-r-4.6.0.tsv").lines() {
        let fields: Vec<_> = row.split('\t').collect();
        assert_eq!(fields.len(), 5);
        let mut frontend = Frontend::default();
        let evaluation = frontend.eval_source(&format!("withVisible({{{}}})", fields[0]));
        match fields[1] {
            "OK" => {
                let values = evaluation.unwrap_or_else(|error| panic!("{}: {error:?}", fields[0]));
                let [Sexp::List(result)] = values.as_slice() else {
                    panic!("{values:?}")
                };
                assert_eq!(encode(&result[0].value), fields[2], "{}", fields[0]);
                assert_eq!(
                    result[1].value,
                    Sexp::Logical(vec![Some(fields[3] == "TRUE")]),
                    "{}",
                    fields[0]
                );
            }
            "ERR" => {
                let error = evaluation.expect_err(fields[0]);
                let payload = match error {
                    EvalError::Arity(message)
                    | EvalError::Domain(message)
                    | EvalError::Type(message)
                    | EvalError::Control(message)
                    | EvalError::Stop { message, .. } => message,
                    other => panic!("unexpected error for {}: {other:?}", fields[0]),
                };
                assert_eq!(payload, fields[2], "{}", fields[0]);
            }
            other => panic!("unexpected fixture status {other}"),
        }
        let output = frontend
            .runtime_mut()
            .take_output()
            .into_iter()
            .map(|event| match event {
                OutputEvent::Stdout(text) => text,
                other => panic!("unexpected event {other:?}"),
            })
            .collect::<String>();
        let hex = output
            .bytes()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        assert_eq!(hex, fields[4], "{}", fields[0]);
    }
}

fn encode(value: &Sexp) -> String {
    match value {
        Sexp::Nil => "NULL".into(),
        Sexp::Primitive(id) => format!(
            "{}:.Primitive(\"{}\")",
            format!("{:?}", id.sexptype()).to_lowercase(),
            id.name()
        ),
        Sexp::Logical(values) => format!(
            "logical:{}",
            values
                .iter()
                .map(|value| match value {
                    Some(true) => "TRUE",
                    Some(false) => "FALSE",
                    None => "NA",
                })
                .collect::<Vec<_>>()
                .join(",")
        ),
        Sexp::Integer(values) => format!(
            "integer:{}",
            values
                .iter()
                .map(|value| value.map_or("NA".into(), |value| value.to_string()))
                .collect::<Vec<_>>()
                .join(",")
        ),
        Sexp::Real(values) => format!(
            "double:{}",
            values
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(",")
        ),
        Sexp::String(values) => format!(
            "character:{}",
            values
                .iter()
                .map(|value| value.as_deref().unwrap_or("NA"))
                .collect::<Vec<_>>()
                .join(",")
        ),
        other => panic!("unrepresented fixture value {other:?}"),
    }
}

#[test]
fn primitive_display_preserves_r_print_special_formals() {
    for (source, expected) in [
        ("sum", "function (..., na.rm = FALSE)  .Primitive(\"sum\")"),
        ("length", "function (x)  .Primitive(\"length\")"),
        (
            "invisible",
            "function (x = NULL)  .Primitive(\"invisible\")",
        ),
        ("`return`", ".Primitive(\"return\")"),
        ("missing", "function (x)  .Primitive(\"missing\")"),
    ] {
        let values = Frontend::default().eval_source(source).unwrap();
        assert_eq!(format_sexp(&values[0]).unwrap(), expected);
    }
}

#[test]
fn forcing_an_invisible_callable_promise_preserves_special_call_visibility() {
    let source = "{g<-function(x,fun=invisible(missing))fun(x);withVisible(g())$visible}";
    assert_eq!(
        Frontend::default().eval_source(source).unwrap(),
        vec![Sexp::Logical(vec![Some(true)])]
    );
}

#[test]
fn missing_follows_unforced_promises_forwarded_through_dots() {
    let source =
        "{m<-missing;g<-function(x)m(x);f<-function(...)g(...);h<-function(x)f(x);c(h(),h(2L))}";
    assert_eq!(
        Frontend::default().eval_source(source).unwrap(),
        vec![Sexp::Logical(vec![Some(true), Some(false)])]
    );
}
