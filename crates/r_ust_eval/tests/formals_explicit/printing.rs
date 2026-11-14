use r_ust_eval::{format_sexp, EvalError, Frontend, OutputEvent};
use r_ust_sexp::{ListElement, Sexp};

#[test]
fn formals_pairlists_keep_nested_prefixes_and_blank_missing_slots() {
    let mut frontend = Frontend::default();
    frontend
        .eval_source_printed(concat!(
            "formals(parent.env);formals(environmentName);",
            "list(a=formals(function(x,y=1L)NULL));",
            "formals(function()NULL);formals(sum)"
        ))
        .unwrap();
    assert_eq!(
        frontend.runtime_mut().take_output(),
        [
            OutputEvent::Stdout("$env\n\n\n".into()),
            OutputEvent::Stdout("$env\n\n\n".into()),
            OutputEvent::Stdout("$a\n$a$x\n\n\n$a$y\n[1] 1\n\n\n".into()),
            OutputEvent::Stdout("NULL\n".into()),
            OutputEvent::Stdout("NULL\n".into()),
        ]
    );
    let pairlist = Sexp::pairlist(vec![
        ListElement {
            name: Some("a".into()),
            value: Sexp::pairlist(vec![ListElement {
                name: Some("b".into()),
                value: Sexp::Integer(vec![Some(1)]),
            }]),
        },
        ListElement {
            name: Some("z".into()),
            value: Sexp::Integer(vec![Some(2)]),
        },
    ]);
    assert_eq!(
        format_sexp(&pairlist).unwrap(),
        "$a\n$a$b\n[1] 1\n\n\n$z\n[1] 2\n"
    );
}

#[test]
fn formals_pairlist_tags_use_r_display_escaping_and_keep_ascii_boundary() {
    let mut frontend = Frontend::default();
    frontend
        .eval_source_printed(r#"formals(function(`a b`, `if`=1L, `a\`b`=2L, `.2`=3L, ...) NULL)"#)
        .unwrap();
    assert_eq!(
        frontend.runtime_mut().take_output(),
        [OutputEvent::Stdout(
            "$`a b`\n\n\n$`if`\n[1] 1\n\n$`a`b`\n[1] 2\n\n$`.2`\n[1] 3\n\n$...\n\n\n".into()
        )]
    );
    let tags = Sexp::pairlist(
        ["a\\b", "line\nbreak", "tab\tname", "\"quote\""]
            .into_iter()
            .map(|name| ListElement {
                name: Some(name.into()),
                value: Sexp::MissingValue,
            })
            .collect(),
    );
    assert_eq!(
        format_sexp(&tags).unwrap(),
        "$`a\\\\b`\n\n\n$`line\\nbreak`\n\n\n$`tab\\tname`\n\n\n$`\"quote\"`\n\n"
    );
    let unsupported = Sexp::pairlist(vec![ListElement {
        name: Some("\u{e9}".into()),
        value: Sexp::MissingValue,
    }]);
    assert_eq!(
        format_sexp(&unsupported),
        Err(EvalError::NotImplemented(
            "language printer non-ASCII text profile"
        ))
    );
}
