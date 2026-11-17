use r_ust_eval::{EvalError, Frontend, OutputEvent, RuntimeState};
use r_ust_sexp::{CallArgument, Environment, ListElement, PrimitiveId, Sexp, SexpType};

#[path = "substitute/errors.rs"]
mod errors;
#[path = "substitute/fixtures.rs"]
mod fixtures;
#[path = "substitute/lifetime.rs"]
mod lifetime;
#[path = "substitute/quote.rs"]
mod quote;
#[path = "substitute/sdk.rs"]
mod sdk;

fn value(source: &str) -> Result<Sexp, Box<dyn std::error::Error>> {
    Frontend::default()
        .eval_source(source)?
        .pop()
        .ok_or_else(|| "missing result".into())
}

fn call(expr: Sexp, env: Sexp) -> Sexp {
    Sexp::Call {
        function: Box::new(Sexp::Primitive(PrimitiveId::Substitute)),
        arguments: vec![
            CallArgument::positional(expr),
            CallArgument::positional(env),
        ],
    }
}

#[test]
fn substitute_matches_independent_r_nodes() -> Result<(), Box<dyn std::error::Error>> {
    for row in fixtures::rows() {
        let mut frontend = Frontend::default();
        frontend.eval_source("options(warn=1)")?;
        if !row.setup.is_empty() {
            frontend.eval_source(row.setup)?;
        }
        let actual = frontend.eval_source(row.source)?.pop().ok_or("result")?;
        assert_eq!(actual, row.expected, "{}", row.id);
        let output = frontend.runtime_mut().take_output();
        let expected = if row.id == "substitute_partial_match" {
            vec![
                OutputEvent::Stderr("Warning in substitute(ex = x, en = list(x = 3L)) :\n  partial argument match of 'ex' to 'expr'\n".into()),
                OutputEvent::Stderr("Warning in substitute(ex = x, en = list(x = 3L)) :\n  partial argument match of 'en' to 'env'\n".into()),
            ]
        } else if row.stdout.is_empty() {
            vec![]
        } else {
            vec![OutputEvent::Stdout(row.stdout.into())]
        };
        assert_eq!(output, expected, "{} output", row.id);
    }
    Ok(())
}

#[test]
fn substitute_has_genuine_special_identity() -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(PrimitiveId::Substitute.sexptype(), SexpType::Special);
    assert_eq!(
        value("substitute")?,
        Sexp::Primitive(PrimitiveId::Substitute)
    );
    assert_eq!(
        value("typeof(substitute)")?,
        Sexp::String(vec![Some("special".into())])
    );
    Ok(())
}
