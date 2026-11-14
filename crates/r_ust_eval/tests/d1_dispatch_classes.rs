use r_ust_eval::Frontend;
use r_ust_sexp::Sexp;

const ALL_METHODS: &[&str] = &[
    "NULL",
    "logical",
    "integer",
    "double",
    "character",
    "list",
    "pairlist",
    "factor",
    "ordered",
    "data.frame",
    "matrix",
    "array",
    "environment",
    "function",
    "call",
    "name",
    "numeric",
    "default",
];

fn invoke(subject: &str, methods: &[&str]) -> Result<Sexp, Box<dyn std::error::Error>> {
    let definitions = methods
        .iter()
        .map(|name| format!("g.{name}<-m"))
        .collect::<Vec<_>>()
        .join(";");
    let source = format!(
        "g<-function(x)UseMethod('g');m<-function(x)c(.Method,.Class);{definitions};g({subject})"
    );
    let mut frontend = Frontend::default();
    let value = frontend
        .eval_source(&source)?
        .pop()
        .ok_or("missing result")?;
    assert_eq!(
        frontend.eval_source("sys.nframe()")?.pop(),
        Some(Sexp::Integer(vec![Some(0)]))
    );
    Ok(value)
}

#[test]
fn represented_classes_and_fallbacks_follow_r_dispatch_order(
) -> Result<(), Box<dyn std::error::Error>> {
    let cases: &[(&str, &[&str], &[&str])] = &[
        ("NULL", ALL_METHODS, &["g.NULL", "NULL"]),
        ("TRUE", ALL_METHODS, &["g.logical", "logical"]),
        ("1L", ALL_METHODS, &["g.integer", "integer", "numeric"]),
        ("1", ALL_METHODS, &["g.double", "double", "numeric"]),
        ("'x'", ALL_METHODS, &["g.character", "character"]),
        ("list(1L)", ALL_METHODS, &["g.list", "list"]),
        ("pairlist(a=1L)", ALL_METHODS, &["g.pairlist", "pairlist"]),
        ("factor('a')", ALL_METHODS, &["g.factor", "factor"]),
        (
            "factor('a',ordered=TRUE)",
            ALL_METHODS,
            &["g.ordered", "ordered", "factor"],
        ),
        (
            "data.frame(x=1L)",
            ALL_METHODS,
            &["g.data.frame", "data.frame"],
        ),
        (
            "matrix(1L,1L,1L)",
            ALL_METHODS,
            &["g.matrix", "matrix", "array", "integer", "numeric"],
        ),
        ("new.env()", ALL_METHODS, &["g.environment", "environment"]),
        ("function()1L", ALL_METHODS, &["g.function", "function"]),
        (
            ".Primitive('length')",
            ALL_METHODS,
            &["g.function", "function"],
        ),
        ("quote(f(1L))", ALL_METHODS, &["g.call", "call"]),
        ("quote(x)", ALL_METHODS, &["g.name", "name"]),
        (
            "factor('a',ordered=TRUE)",
            &["factor", "default"],
            &["g.factor", "factor"],
        ),
        (
            "matrix(1L,1L,1L)",
            &["array", "integer", "numeric", "default"],
            &["g.array", "array", "integer", "numeric"],
        ),
        (
            "matrix(1L,1L,1L)",
            &["integer", "numeric", "default"],
            &["g.integer", "integer", "numeric"],
        ),
        ("1L", &["numeric", "default"], &["g.numeric", "numeric"]),
        ("1", &["numeric", "default"], &["g.numeric", "numeric"]),
    ];
    for (subject, methods, expected) in cases {
        assert_eq!(
            invoke(subject, methods)?,
            Sexp::String(expected.iter().map(|text| Some((*text).into())).collect()),
            "{subject}: {methods:?}"
        );
    }
    Ok(())
}

#[test]
fn dispatched_method_keeps_actual_callable_for_alias_and_computed_heads(
) -> Result<(), Box<dyn std::error::Error>> {
    let mut frontend = Frontend::default();
    let value = frontend.eval_source("g<-function(x)UseMethod('g');m<-function(x)identical(sys.function(),m);g.integer<-m;h<-g;c(g(1L),h(1L),(function()g)()(1L))")?.pop();
    assert_eq!(value, Some(Sexp::Logical(vec![Some(true); 3])));
    assert_eq!(
        frontend.eval_source("sys.nframe()")?.pop(),
        Some(Sexp::Integer(vec![Some(0)]))
    );
    Ok(())
}
