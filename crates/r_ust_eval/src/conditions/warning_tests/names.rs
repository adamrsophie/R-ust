use super::{streams, Frontend, Sexp};
use r_ust_sexp::{canonical_actual_call, PrimitiveId};

fn evaluate(frontend: &mut Frontend, source: &str) -> Sexp {
    frontend
        .eval_source(source)
        .unwrap_or_else(|error| panic!("{source}: {error}"))
        .pop()
        .unwrap()
}

#[test]
fn do_call_preserves_missing_literal_blank_and_absent_name_distinctions() {
    let cases = [
        ("absent", "", Sexp::Nil),
        ("blank", "names(a)<-'';", Sexp::Nil),
        (
            "na",
            "names(a)<-NA_character_;",
            Sexp::String(vec![Some("NA".into())]),
        ),
        (
            "literal_na",
            "names(a)<-'NA';",
            Sexp::String(vec![Some("NA".into())]),
        ),
    ];
    for (id, assignment, expected) in cases {
        let mut frontend = Frontend::default();
        let source =
            format!("a<-list(7L);{assignment}f<-function(...)names(list(...));do.call(f,a)");
        assert_eq!(evaluate(&mut frontend, &source), expected, "{id}");
        let value = evaluate(&mut frontend, "do.call(list,a)");
        let projection = crate::ordinary_values::read(&value).unwrap();
        let view = projection.as_ref().ordinary_list().unwrap();
        assert_eq!(view.names_present, !matches!(expected, Sexp::Nil), "{id}");
        assert_eq!(view.elements.len(), 1, "{id}");
        assert_eq!(
            view.elements[0].name.as_deref(),
            if matches!(expected, Sexp::Nil) {
                None
            } else {
                Some("NA")
            },
            "{id}"
        );
        assert_eq!(view.elements[0].value, Sexp::Integer(vec![Some(7)]), "{id}");
        if id == "na" {
            assert_eq!(
                evaluate(&mut frontend, "names(a)"),
                Sexp::String(vec![None]),
                "input remains NA"
            );
        }
        assert_eq!(
            streams(&mut frontend),
            (String::new(), String::new()),
            "{id}"
        );
    }
    let mut frontend = Frontend::default();
    assert_eq!(evaluate(&mut frontend, "a<-list(1L,2L,3L,4L);names(a)<-c('',NA_character_,'NA','x');f<-function(...)names(list(...));do.call(f,a)"),
        strings(&["", "NA", "NA", "x"]));
    assert_eq!(
        evaluate(&mut frontend, "names(a)"),
        Sexp::String(vec![
            Some("".into()),
            None,
            Some("NA".into()),
            Some("x".into())
        ])
    );
    let value = evaluate(&mut frontend, "do.call(list,a)");
    let projection = crate::ordinary_values::read(&value).unwrap();
    let view = projection.as_ref().ordinary_list().unwrap();
    assert!(view.names_present);
    assert_eq!(
        view.elements
            .iter()
            .map(|e| e.name.as_deref())
            .collect::<Vec<_>>(),
        vec![Some(""), Some("NA"), Some("NA"), Some("x")]
    );
    assert_eq!(
        evaluate(
            &mut frontend,
            "a<-list(7L);names(a)<-NA_character_;f<-function(`NA`)`NA`;do.call(f,a)"
        ),
        Sexp::Integer(vec![Some(7)])
    );
    assert_eq!(evaluate(&mut frontend, "a<-list(1L,2L);names(a)<-c(NA_character_,'NA');f<-function(`NA`,...)`NA`;tryCatch(do.call(f,a),error=function(e)conditionMessage(e))"),
        strings(&["formal argument \"NA\" matched by multiple actual arguments"]));
    assert_eq!(streams(&mut frontend), (String::new(), String::new()));
}

#[test]
fn as_call_list_and_expression_names_become_exact_tags_and_drop_head_tag() {
    for constructor in ["list", "expression"] {
        for (id, assignment, tag) in [
            ("absent", "", None),
            ("blank", "names(a)<-c('','');", None),
            (
                "na",
                "names(a)<-c(NA_character_,NA_character_);",
                Some("NA"),
            ),
            ("literal_na", "names(a)<-c('NA','NA');", Some("NA")),
            ("head_only", "names(a)<-c('head','');", None),
        ] {
            let mut frontend = Frontend::default();
            let source = format!("a<-{constructor}(sum,7L);{assignment}z<-as.call(a);z");
            let result = evaluate(&mut frontend, &source);
            let Sexp::Call {
                function,
                arguments,
            } = canonical_actual_call(&result).unwrap()
            else {
                panic!("{constructor} {id}");
            };
            let expected_head = if constructor == "list" {
                Sexp::Primitive(PrimitiveId::Sum)
            } else {
                Sexp::Symbol("sum".into())
            };
            assert_eq!(*function, expected_head, "{constructor} {id}");
            assert_eq!(arguments.len(), 1, "{constructor} {id}");
            assert_eq!(arguments[0].name.as_deref(), tag, "{constructor} {id}");
            assert_eq!(arguments[0].value, Sexp::Integer(vec![Some(7)]));
            assert_eq!(
                evaluate(&mut frontend, "names(z)"),
                if tag.is_some() {
                    strings(&["", "NA"])
                } else {
                    Sexp::Nil
                },
                "{constructor} {id}"
            );
            assert_eq!(
                evaluate(&mut frontend, "eval(z)"),
                Sexp::Integer(vec![Some(7)]),
                "{constructor} {id}"
            );
            if id == "na" {
                assert_eq!(
                    evaluate(&mut frontend, "names(a)"),
                    Sexp::String(vec![None, None])
                );
            }
            assert_eq!(streams(&mut frontend), (String::new(), String::new()));
        }
        let mut frontend = Frontend::default();
        let source = format!("a<-{constructor}(sum,1L,2L,3L,4L);names(a)<-c('head','',NA_character_,'NA','x');z<-as.call(a);z");
        let result = evaluate(&mut frontend, &source);
        let Sexp::Call { arguments, .. } = canonical_actual_call(&result).unwrap() else {
            panic!("mixed call");
        };
        assert_eq!(
            arguments
                .iter()
                .map(|a| a.name.as_deref())
                .collect::<Vec<_>>(),
            vec![None, Some("NA"), Some("NA"), Some("x")]
        );
        assert_eq!(
            evaluate(&mut frontend, "names(z)"),
            strings(&["", "", "NA", "NA", "x"])
        );
        assert_eq!(
            evaluate(&mut frontend, "eval(z)"),
            Sexp::Integer(vec![Some(10)])
        );
        assert_eq!(streams(&mut frontend), (String::new(), String::new()));
    }
}

fn strings(values: &[&str]) -> Sexp {
    Sexp::String(values.iter().map(|value| Some((*value).into())).collect())
}
