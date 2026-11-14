use r_ust_eval::{format_sexp, Frontend};
use r_ust_parse::Parser;
use r_ust_sexp::{canonical_language, CallArgument, ClosureEnvironment, Sexp};

const STRUCTURE: &str = include_str!("structure.R");

#[test]
fn exact_structure_parses_and_inspected_body_round_trips() {
    let parsed = Parser.parse_all(STRUCTURE).unwrap();
    let [Sexp::Assignment { value, .. }] = parsed.expressions.as_slice() else {
        panic!("source assignment");
    };
    let Sexp::Function { body, .. } = value.as_ref() else {
        panic!("source function");
    };
    let expected = canonical_language(body).unwrap();
    let mut frontend = Frontend::default();
    assert_eq!(
        frontend.eval_source("body(structure)").unwrap().as_slice(),
        std::slice::from_ref(&expected)
    );
    let printed = format_sexp(&expected).unwrap();
    assert!(printed.contains("exact = TRUE"));
    assert!(printed.contains("names(attrib)[ok] <-"));
    assert!(printed.contains("storage.mode(.Data) <-"));
    let round_trip = Parser.parse_all(&printed).unwrap();
    assert_eq!(
        canonical_language(&round_trip.expressions[0]).unwrap(),
        expected
    );
}

#[test]
fn additive_syntax_preserves_names_missingness_and_legacy_constructors() {
    for source in [
        "x[['a',exact=TRUE]]",
        "x[[,exact=TRUE]]",
        "attributes(x)<-list(class='a')",
        "names(x)[ok]<-y",
        "storage.mode(x)<-'integer'",
    ] {
        let parsed = Parser.parse_all(source).unwrap();
        let canonical = canonical_language(&parsed.expressions[0]).unwrap();
        let printed = format_sexp(&canonical).unwrap();
        let reparsed = Parser.parse_all(&printed).unwrap();
        assert_eq!(
            canonical_language(&reparsed.expressions[0]).unwrap(),
            canonical,
            "{source}"
        );
    }
    let old = Sexp::Extract {
        value: Box::new(Sexp::Symbol("x".into())),
        index: Box::new(Sexp::Integer(vec![Some(1)])),
    };
    assert_eq!(Parser.parse_all("x[[1L]]").unwrap().expressions, [old]);
    let runtime_list = Sexp::ClassedList {
        elements: vec![],
        class: Box::new(Sexp::String(vec![Some("box".into())])),
        class_first: true,
    };
    assert!(canonical_language(&runtime_list).is_err());
    let mut frontend = Frontend::default();
    assert!(frontend
        .runtime_mut()
        .eval(&Sexp::Call {
            function: Box::new(Sexp::Symbol("quote".into())),
            arguments: vec![CallArgument::positional(runtime_list)]
        })
        .is_err());
}

#[test]
fn source_templates_are_real_closures() {
    let mut frontend = Frontend::default();
    for name in ["structure", "pairlist", "as.pairlist"] {
        let values = frontend.eval_source(name).unwrap();
        let [Sexp::Closure {
            body,
            environment: ClosureEnvironment::Captured(_),
            ..
        }] = values.as_slice()
        else {
            panic!("{name} is not a closure");
        };
        assert!(canonical_language(body).is_ok());
    }
}

#[test]
fn all_forthcoming_source_closures_parse_without_activation() {
    let parsed = Parser.parse_all(include_str!("forthcoming.R")).unwrap();
    let mut names = Vec::new();
    for expression in parsed.expressions {
        let Sexp::Assignment { target, value, .. } = expression else {
            panic!("source assignment");
        };
        let Sexp::Function { parameters, body } = value.as_ref() else {
            panic!("source closure");
        };
        assert!(canonical_language(body).is_ok(), "{target}");
        for parameter in parameters {
            if let Some(default) = &parameter.default {
                assert!(canonical_language(default).is_ok());
            }
        }
        names.push(target);
    }
    assert_eq!(
        names,
        [
            "print",
            "print.default",
            "tryCatch",
            "withCallingHandlers",
            "simpleError",
            "conditionMessage",
            "conditionCall",
            "conditionMessage.condition",
            "conditionCall.condition",
            ".handleSimpleError"
        ]
    );
}

#[test]
fn named_extraction_validates_indices_and_preserves_evaluation_order() {
    let mut frontend = Frontend::default();
    frontend.eval_source("x<-list(a=1L)").unwrap();
    for (source, message) in [
        ("x[[exact=TRUE]]", "no index specified"),
        (
            "x[['a',exact=TRUE,exact=TRUE]]",
            "incorrect number of subscripts",
        ),
    ] {
        assert_eq!(
            frontend.eval_source(source),
            Err(r_ust_eval::EvalError::Domain(message.into()))
        );
    }
    let values = frontend
        .eval_source("({cat('TARGET');x})[[{cat('INDEX');'a'},exact={cat('EXACT');TRUE}]]")
        .unwrap();
    assert_eq!(values, [Sexp::Integer(vec![Some(1)])]);
    assert_eq!(
        frontend.runtime_mut().take_output(),
        [
            r_ust_eval::OutputEvent::Stdout("TARGET".into()),
            r_ust_eval::OutputEvent::Stdout("INDEX".into()),
            r_ust_eval::OutputEvent::Stdout("EXACT".into())
        ]
    );
    frontend.eval_source("`[[`<-function(x,...)9L").unwrap();
    assert_eq!(
        frontend.eval_source("x[['a',exact=TRUE]]").unwrap(),
        [Sexp::Integer(vec![Some(9)])]
    );
}
