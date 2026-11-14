use super::*;

pub(super) fn ordinary_observer_boundary() -> Result {
    let mut frontend = Frontend::default();
    assert_eq!(
        frontend.eval_source(include_str!("ordinary-frozen.R")),
        Err(EvalError::UndefinedFunction("dput".into()))
    );
    let previous = || {
        vec![attribute(
            "previous",
            strings(&[Some("a"), Some("b"), Some("c")]),
        )]
    };
    let original = || attributed(&[Some("b"), Some("c")], previous());
    let assigned = || attributed(&[Some("z"), Some("c")], previous());
    let expected = Sexp::List(vec![
        attribute("original", original()),
        attribute("subset", strings(&[Some("b")])),
        attribute("subset.all", original()),
        attribute("extract", strings(&[Some("b")])),
        attribute("character", strings(&[Some("b"), Some("c")])),
        attribute("combined", strings(&[Some("b"), Some("c")])),
        attribute(
            "repeated",
            strings(&[Some("b"), Some("c"), Some("b"), Some("c")]),
        ),
        attribute("subset.assign", assigned()),
        attribute("extract.assign", assigned()),
        attribute("plain.equal", Sexp::Logical(vec![Some(false)])),
        attribute("same.equal", Sexp::Logical(vec![Some(true)])),
    ]);
    assert_eq!(frontend.runtime().get("ans")?, Some(expected));
    println!(
        "frozen ordinary-character construction verified; literal execution remains PENDING_DPUT"
    );
    Ok(())
}
