use r_ust_eval::{Frontend, OutputEvent};
use r_ust_sexp::Sexp;

fn last(source: &str) -> Result<Sexp, Box<dyn std::error::Error>> {
    let mut runtime = Frontend::default();
    runtime
        .eval_source(source)?
        .pop()
        .ok_or_else(|| "missing result".into())
}

#[test]
fn original_actual_survives_local_reassignment_and_method_forcing(
) -> Result<(), Box<dyn std::error::Error>> {
    let value = last("n <- 0L; g <- function(x) { x <- 'changed'; UseMethod('g'); 99L }; g.integer <- function(x) { x; x; n }; g({ n <<- n + 1L; 1L })")?;
    assert_eq!(value, Sexp::Integer(vec![Some(1)]));
    Ok(())
}

#[test]
fn method_receives_full_previous_class_value() -> Result<(), Box<dyn std::error::Error>> {
    let value = last("cl <- structure(c('a', 'b'), note='retained'); x <- structure(list(), class=cl); g <- function(x) UseMethod('g'); g.b <- function(x) identical(attr(.Class, 'previous'), cl); g(x)")?;
    assert_eq!(value, Sexp::Logical(vec![Some(true)]));
    Ok(())
}

#[test]
fn default_method_gets_null_class_and_its_own_default() -> Result<(), Box<dyn std::error::Error>> {
    let value = last("g <- function(x=stop('generic default forced')) UseMethod('g'); g.default <- function(x=17L) { if (is.null(.Class)) x else 99L }; g()")?;
    assert_eq!(value, Sexp::Integer(vec![Some(17)]));
    Ok(())
}

#[test]
fn implicit_numeric_suffix_preserves_type_class() -> Result<(), Box<dyn std::error::Error>> {
    for (expression, expected) in [("1L", "integer"), ("1", "double")] {
        let value = last(&format!("g <- function(x) UseMethod('g'); g.numeric <- function(x) attr(.Class, 'previous'); g({expression})"))?;
        assert_eq!(
            value,
            Sexp::String(vec![Some(expected.into()), Some("numeric".into())])
        );
    }
    Ok(())
}

#[test]
fn invalid_generic_does_not_force_object_and_restores_stack(
) -> Result<(), Box<dyn std::error::Error>> {
    let mut runtime = Frontend::default();
    runtime.eval_source("n <- 0L; g <- function(x) UseMethod(1, { n <<- 1L; x })")?;
    let error = runtime
        .eval_source("g(1L)")
        .expect_err("invalid generic must fail");
    assert!(error
        .to_string()
        .contains("'generic' argument must be a character string"));
    assert_eq!(
        runtime.eval_source("c(n, sys.nframe())")?.pop(),
        Some(Sexp::Integer(vec![Some(0), Some(0)]))
    );
    Ok(())
}

#[test]
fn method_sysparent_inherits_generic_caller() -> Result<(), Box<dyn std::error::Error>> {
    let value = last("g <- function(x) UseMethod('g'); g.integer <- function(x) sys.parent(); f <- function() g(1L); f()")?;
    assert_eq!(value, Sexp::Integer(vec![Some(1)]));
    Ok(())
}

#[test]
fn alias_and_computed_print_heads_dispatch_with_method_visibility(
) -> Result<(), Box<dyn std::error::Error>> {
    for call in ["p(1L)", "(p)(1L)", "(function() p)()(1L)"] {
        let value = last(&format!("p <- print; print.integer <- function(x) invisible(27L); r <- withVisible({call}); identical(r$value, 27L) && identical(r$visible, FALSE)"))?;
        assert_eq!(value, Sexp::Logical(vec![Some(true)]));
    }
    Ok(())
}

#[test]
fn default_printer_applies_quote_and_preserves_invisibility(
) -> Result<(), Box<dyn std::error::Error>> {
    let mut frontend = Frontend::default();
    let value = frontend.eval_source("r <- withVisible(print('word', quote=FALSE)); r$visible")?;
    assert_eq!(value.last(), Some(&Sexp::Logical(vec![Some(false)])));
    let output: String = frontend
        .runtime_mut()
        .take_output()
        .into_iter()
        .filter_map(|event| match event {
            OutputEvent::Stdout(text) => Some(text),
            _ => None,
        })
        .collect();
    assert_eq!(output, "[1] word\n");
    Ok(())
}

#[test]
fn printer_rejects_unsupported_controls_in_source_order() -> Result<(), Box<dyn std::error::Error>>
{
    let mut frontend = Frontend::default();
    let error = frontend
        .eval_source("print(1L, digits=3L, quote=NA)")
        .expect_err("unsupported digits must fail before quote");
    assert!(error.to_string().contains("print.default digits"));
    assert_eq!(
        frontend.eval_source("sys.nframe()")?.last(),
        Some(&Sexp::Integer(vec![Some(0)]))
    );
    Ok(())
}
