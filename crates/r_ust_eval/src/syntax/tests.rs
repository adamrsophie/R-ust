use crate::{CallArgument, EvalError, EvalFlow, ListElement, RuntimeState, Sexp};
use r_ust_sexp::{canonical_language, PrimitiveId, SyntaxPrimitive};

mod control;
mod equivalence;
mod identity;
mod shapes;

fn integer(value: i32) -> Sexp {
    Sexp::Integer(vec![Some(value)])
}
fn logical(value: bool) -> Sexp {
    Sexp::Logical(vec![Some(value)])
}
fn symbol(name: &str) -> Sexp {
    Sexp::Symbol(name.into())
}
fn call(name: &str, values: Vec<Sexp>) -> Sexp {
    Sexp::Call {
        function: Box::new(symbol(name)),
        arguments: values.into_iter().map(CallArgument::positional).collect(),
    }
}
fn expression(source: &str) -> Sexp {
    let mut values = crate::Parser.parse_all(source).unwrap().expressions;
    assert_eq!(values.len(), 1);
    values.remove(0)
}
fn source(runtime: &mut RuntimeState, text: &str, canonical: bool) -> Sexp {
    let mut result = Sexp::Nil;
    for expression in crate::Parser.parse_all(text).unwrap().expressions {
        let expression = if canonical {
            canonical_language(&expression).unwrap()
        } else {
            expression
        };
        result = runtime.eval(&expression).unwrap();
    }
    result
}
fn check_value(actual: &Sexp, expected: &Sexp, id: &str) {
    assert!(
        crate::sexp_identical(actual, expected).unwrap(),
        "{id}: actual {actual:?}, expected {expected:?}"
    );
}
