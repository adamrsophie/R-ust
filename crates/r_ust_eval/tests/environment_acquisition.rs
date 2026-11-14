use r_ust_eval::{Frontend, OutputEvent};
use r_ust_sexp::Sexp;

#[path = "environment_acquisition/arguments.rs"]
mod arguments;
#[path = "environment_acquisition/source.rs"]
mod source;

#[test]
fn named_roots_follow_the_real_session_chain() {
    let mut frontend = Frontend::default();
    let values = frontend
        .eval_source(concat!(
            "identical(globalenv(), .GlobalEnv); ",
            "identical(parent.env(globalenv()), .AutoloadEnv); ",
            "identical(parent.env(.AutoloadEnv), baseenv()); ",
            "identical(parent.env(baseenv()), emptyenv()); ",
            "identical(parent.env(.BaseNamespaceEnv), globalenv())"
        ))
        .unwrap();
    assert_eq!(values, vec![Sexp::Logical(vec![Some(true)]); 5]);
}

#[test]
fn named_roots_print_visible_values_after_invisible_arguments() {
    let mut frontend = Frontend::default();
    frontend
        .eval_source_printed(concat!(
            "globalenv(); baseenv(); emptyenv(); ",
            "is.environment(invisible(globalenv())); ",
            "environmentName(invisible(baseenv()))"
        ))
        .unwrap();
    assert_eq!(
        frontend.runtime_mut().take_output(),
        vec![
            OutputEvent::Stdout("<environment: R_GlobalEnv>\n".into()),
            OutputEvent::Stdout("<environment: base>\n".into()),
            OutputEvent::Stdout("<environment: R_EmptyEnv>\n".into()),
            OutputEvent::Stdout("[1] TRUE\n".into()),
            OutputEvent::Stdout("[1] \"base\"\n".into()),
        ]
    );
}

#[test]
fn named_roots_aliases_computed_heads_and_primitive_metadata_agree() {
    let mut frontend = Frontend::default();
    for (name, printed) in [
        ("globalenv", "<environment: R_GlobalEnv>"),
        ("baseenv", "<environment: base>"),
        ("emptyenv", "<environment: R_EmptyEnv>"),
    ] {
        frontend
            .eval_source_printed(&format!(
                "f<-{name}; f(); (identity(f))(); .Primitive('{name}')(); typeof(f); f"
            ))
            .unwrap();
        assert_eq!(
            frontend.runtime_mut().take_output(),
            [
                OutputEvent::Stdout(format!("{printed}\n")),
                OutputEvent::Stdout(format!("{printed}\n")),
                OutputEvent::Stdout(format!("{printed}\n")),
                OutputEvent::Stdout("[1] \"builtin\"\n".into()),
                OutputEvent::Stdout(format!("function ()  .Primitive(\"{name}\")\n")),
            ],
            "{name}"
        );
    }
    frontend
        .eval_source_printed(concat!(
            "p<-is.environment; p(x=globalenv()); (identity(p))(1L);",
            ".Primitive('is.environment')(emptyenv()); p"
        ))
        .unwrap();
    assert_eq!(
        frontend.runtime_mut().take_output(),
        [
            OutputEvent::Stdout("[1] TRUE\n".into()),
            OutputEvent::Stdout("[1] FALSE\n".into()),
            OutputEvent::Stdout("[1] TRUE\n".into()),
            OutputEvent::Stdout("function (x)  .Primitive(\"is.environment\")\n".into()),
        ]
    );
}

#[test]
fn environment_names_distinguish_roots_and_non_environments() {
    let mut frontend = Frontend::default();
    for (source, name) in [
        ("globalenv()", "R_GlobalEnv"),
        ("baseenv()", "base"),
        ("emptyenv()", "R_EmptyEnv"),
        (".BaseNamespaceEnv", "base"),
        (".AutoloadEnv", "Autoloads"),
        ("1L", ""),
        ("NULL", ""),
        ("list()", ""),
        ("identity", ""),
        ("sum", ""),
        ("quote(x)", ""),
        ("formals(identity)", ""),
    ] {
        assert_eq!(
            frontend
                .eval_source(&format!("environmentName({source})"))
                .unwrap(),
            [Sexp::String(vec![Some(name.into())])],
            "{source}"
        );
        assert_eq!(
            frontend
                .eval_source(&format!("is.environment({source})"))
                .unwrap(),
            [Sexp::Logical(vec![Some(!name.is_empty())])],
            "{source}"
        );
    }
    for name in ["parent.env", "environmentName"] {
        assert_eq!(
            frontend.eval_source(&format!("typeof({name})")).unwrap(),
            [Sexp::String(vec![Some("closure".into())])]
        );
        assert_eq!(
            frontend.eval_source(&format!(".Primitive('{name}')")),
            Err(r_ust_eval::EvalError::Domain(
                "no such primitive function".into()
            ))
        );
    }
}
