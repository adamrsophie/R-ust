use r_ust_eval::{Frontend, OutputEvent, RuntimeState};
use r_ust_sexp::{CallArgument, Environment, ListElement, PrimitiveId, Sexp, SexpType};

#[path = "expression_values/fixtures.rs"]
mod fixtures;
#[path = "expression_values/sdk.rs"]
mod sdk;

fn value(source: &str) -> Result<Sexp, Box<dyn std::error::Error>> {
    Frontend::default()
        .eval_source(source)?
        .pop()
        .ok_or_else(|| "missing result".into())
}

#[test]
fn expression_values_match_independent_r_nodes() -> Result<(), Box<dyn std::error::Error>> {
    for row in fixtures::rows() {
        let mut frontend = Frontend::default();
        if !row.setup.is_empty() {
            frontend.eval_source(row.setup)?;
        }
        let actual = frontend.eval_source(row.source)?.pop().ok_or("result")?;
        assert_eq!(actual, row.expected, "{}", row.id);
        let output = frontend.runtime_mut().take_output();
        let expected = if row.stdout.is_empty() {
            vec![]
        } else {
            vec![OutputEvent::Stdout(row.stdout.into())]
        };
        assert_eq!(output, expected, "{} output", row.id);
    }
    Ok(())
}

#[test]
fn expression_is_first_class_special_and_aliases_capture_without_forcing(
) -> Result<(), Box<dyn std::error::Error>> {
    let expected = value("expression(a=stop('BAD'),,x+1L)")?;
    for source in [
        "e<-expression;e(a=stop('BAD'),,x+1L)",
        "(identity(expression))(a=stop('BAD'),,x+1L)",
        ".Primitive('expression')(a=stop('BAD'),,x+1L)",
    ] {
        assert_eq!(value(source)?, expected, "{source}");
    }
    assert_eq!(PrimitiveId::Expression.sexptype(), SexpType::Special);
    assert_eq!(
        value("expression")?,
        Sexp::Primitive(PrimitiveId::Expression)
    );
    assert_eq!(
        value("typeof(expression)")?,
        Sexp::String(vec![Some("special".into())])
    );
    assert_eq!(
        value("typeof(.Primitive('is.expression'))")?,
        Sexp::String(vec![Some("builtin".into())])
    );
    Ok(())
}

#[test]
fn expression_type_names_and_numeric_identity_stay_distinct(
) -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(
        value("identical(expression(x),list(quote(x)))")?,
        Sexp::Logical(vec![Some(false)])
    );
    assert_eq!(
        value("identical(expression(NA_real_),expression(NaN))")?,
        Sexp::Logical(vec![Some(false)])
    );
    assert_eq!(
        value("identical(expression(NaN),expression(NaN))")?,
        Sexp::Logical(vec![Some(true)])
    );
    assert_eq!(
        value("a<-expression();b<-expression();names(b)<-character();identical(a,b)")?,
        Sexp::Logical(vec![Some(false)])
    );
    assert_eq!(
        value("e<-expression(x,y);names(e)<-c(NA_character_,'');names(e)")?,
        Sexp::String(vec![None, Some(String::new())])
    );
    assert_eq!(
        value("e<-expression(x,y);names(e)<-'a';names(e)")?,
        Sexp::String(vec![Some("a".into()), None])
    );
    Ok(())
}

#[test]
fn expression_subset_preserves_holes_names_and_null_oob() -> Result<(), Box<dyn std::error::Error>>
{
    let actual = value("expression(a=x,y)[c(2L,3L,NA_integer_)]")?;
    assert_eq!(
        actual,
        Sexp::Expression {
            elements: vec![
                ListElement {
                    name: Some(String::new()),
                    value: Sexp::Symbol("y".into())
                },
                ListElement {
                    name: None,
                    value: Sexp::Nil
                },
                ListElement {
                    name: None,
                    value: Sexp::Nil
                },
            ],
            names_present: true,
        }
    );
    assert_eq!(value("expression(a=x)[['a']]")?, Sexp::Symbol("x".into()));
    assert_eq!(value("expression(a=x)[['absent']]")?, Sexp::Nil);
    assert_eq!(value("expression(a=x)[['']]")?, Sexp::Nil);
    Ok(())
}

#[test]
fn expression_prints_exact_names_holes_and_nested_containers(
) -> Result<(), Box<dyn std::error::Error>> {
    for (source, expected) in [
        ("expression()", "expression()\n"),
        ("list(expression(x))", "[[1]]\nexpression(x)\n\n"),
        ("expression(a=,)", "expression(a = , )\n"),
        ("expression(,)", "expression(, )\n"),
        (
            "e<-expression(x,y);names(e)<-c(NA_character_,'');e",
            "expression(`NA` = x, y)\n",
        ),
    ] {
        let result = value(source)?;
        let mut frontend = Frontend::default();
        frontend.eval_printed(&result)?;
        assert_eq!(
            frontend.runtime_mut().take_output(),
            vec![OutputEvent::Stdout(expected.into())]
        );
    }
    Ok(())
}

#[test]
fn expression_iteration_passes_data_without_sequencing() -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(
        value("for(x in expression(stop('BAD'))) y<-x; identical(y,quote(stop('BAD')))")?,
        Sexp::Logical(vec![Some(true)])
    );
    assert_eq!(value("c(is.vector(expression(x)),is.vector(expression(x),'expression'),is.vector(expression(x),'list'))")?, Sexp::Logical(vec![Some(true),Some(true),Some(false)]));
    Ok(())
}
