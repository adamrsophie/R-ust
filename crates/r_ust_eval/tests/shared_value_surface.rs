use r_ust_eval::{format_sexp, EvalError, Frontend, RuntimeState};
use r_ust_sexp::{AssignmentOperator, CallArgument, Environment, ListElement, ListObject, Sexp};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[path = "shared_values/pairlist.rs"]
mod pairlist;

#[path = "shared_values/attributes.rs"]
mod attributes;

#[path = "shared_values/char_getters.rs"]
mod char_getters;

fn bind(runtime: &mut RuntimeState, value: Sexp) -> Result {
    runtime.eval(&Sexp::Assignment {
        operator: AssignmentOperator::Left,
        target: "x".into(),
        value: Box::new(value),
    })?;
    Ok(())
}

fn call(runtime: &mut RuntimeState, name: &str, values: Vec<Sexp>) -> Result<Sexp> {
    Ok(runtime.eval(&Sexp::Call {
        function: Box::new(Sexp::Symbol(name.into())),
        arguments: values.into_iter().map(CallArgument::positional).collect(),
    })?)
}

fn payload() -> Sexp {
    Sexp::List(vec![
        ListElement {
            name: Some("a".into()),
            value: Sexp::Integer(vec![Some(1)]),
        },
        ListElement {
            name: Some("b".into()),
            value: Sexp::Integer(vec![Some(2)]),
        },
    ])
}

fn char_expression(value: Sexp) -> Sexp {
    Sexp::Extract {
        value: Box::new(Sexp::List(vec![ListElement { name: None, value }])),
        index: Box::new(Sexp::Integer(vec![Some(1)])),
    }
}

#[test]
fn ordinary_surfaces_and_cow() -> Result {
    let owner = Environment::global();
    for source in [
        "typeof(x)",
        "mode(x)",
        "class(x)",
        "length(x)",
        "names(x)",
        "attributes(x)",
        "is.object(x)",
        "is.list(x)",
        "is.vector(x)",
        "is.atomic(x)",
        "is.na(x)",
        "anyNA(x)",
        "x[1L]",
        "x['a']",
        "x[[1L]]",
        "x$a",
        "lengths(x)",
        "as.integer(x)",
        "as.logical(x)",
        "as.character(x)",
        "unlist(x)",
        "as.list(x)",
        "as.pairlist(x)",
        "c(x,list(c=3L))",
        "rep(x,2L)",
        "rep_len(x,3L)",
        "lapply(x,identity)",
        "rev(x)",
        "head(x,1L)",
        "tail(x,1L)",
        "unique(x)",
        "duplicated(x)",
        "matrix(x,1L,2L)",
        "print(x)",
        "identical(x,list(a=1L,b=2L))",
    ] {
        let graph = ListObject::from_value(&owner, &payload())?;
        graph.set_gp_shared(7)?;
        let mut inline = Frontend::default();
        let mut shared = Frontend::default();
        bind(inline.runtime_mut(), payload())?;
        bind(shared.runtime_mut(), Sexp::ListObject(graph.clone()))?;
        let expected = inline.eval_source(source)?;
        let actual = shared.eval_source(source)?;
        assert_eq!(
            inline.runtime().is_visible(),
            shared.runtime().is_visible(),
            "{source}"
        );
        assert_eq!(
            inline.runtime_mut().take_output(),
            shared.runtime_mut().take_output(),
            "{source}"
        );
        assert_eq!(actual.len(), expected.len(), "{source}");
        for (actual, expected) in actual.into_iter().zip(expected) {
            assert_eq!(
                call(shared.runtime_mut(), "identical", vec![actual, expected])?,
                Sexp::Logical(vec![Some(true)]),
                "{source}"
            );
        }
        assert_eq!(graph.header()?.gp(), 7);
    }
    for source in [
        "x[1L]<-list(9L)",
        "x[[1L]]<-9L",
        "x[[1L]]<-NULL",
        "x[[4L]]<-9L",
        "x$a<-9L",
        "x$a<-NULL",
        "x$c<-9L",
        "names(x)<-c('c','d')",
        "class(x)<-'box'",
        "attributes(x)<-list(names=c('c','d'))",
    ] {
        let graph = ListObject::from_value(&owner, &payload())?;
        graph.set_gp_shared(7)?;
        let mut frontend = Frontend::default();
        bind(frontend.runtime_mut(), Sexp::ListObject(graph.clone()))?;
        frontend.eval_source(source)?;
        let values = frontend.eval_source("x")?;
        let [Sexp::ListObject(copy)] = values.as_slice() else {
            panic!("{source}: graph storage lost")
        };
        assert!(!copy.same_object(&graph), "{source}");
        assert_eq!(copy.header()?.gp(), 7, "{source}");
        assert_eq!(graph.value()?, payload(), "{source}");
    }
    Ok(())
}

#[test]
fn char_and_recursive_boundaries() -> Result {
    let mut runtime = RuntimeState::default();
    for (value, bytes, printed) in [
        (Some("error"), 5, "<CHARSXP: \"error\">"),
        (Some("é"), 2, "<CHARSXP: \"<U+00E9>\">"),
        (None, 2, "<CHARSXP: NA>"),
        (Some("NA"), 2, "<CHARSXP: \"NA\">"),
    ] {
        let value = Sexp::Char(value.map(str::to_owned));
        assert_eq!(format_sexp(&value)?, printed);
        assert_eq!(
            call(&mut runtime, "typeof", vec![char_expression(value.clone())])?,
            Sexp::String(vec![Some("char".into())])
        );
        assert_eq!(
            call(&mut runtime, "length", vec![char_expression(value.clone())])?,
            Sexp::Integer(vec![Some(bytes)])
        );
        assert_eq!(
            call(
                &mut runtime,
                "identical",
                vec![char_expression(value), Sexp::String(vec![None])]
            )?,
            Sexp::Logical(vec![Some(false)])
        );
    }
    assert_eq!(
        runtime.eval(&Sexp::Char(Some("error".into()))),
        Err(EvalError::Type(
            "unimplemented type 'char' in 'eval'\n".into()
        ))
    );
    assert!(runtime.eval(&Sexp::Char(Some("a\0b".into()))).is_err());
    let owner = Environment::global();
    let a = ListObject::from_value(&owner, &payload())?;
    let b = ListObject::from_value(&owner, &payload())?;
    a.replace_shared(&[(0, Sexp::ListObject(a.clone()))])?;
    b.replace_shared(&[(0, Sexp::ListObject(b.clone()))])?;
    assert_eq!(
        format_sexp(&Sexp::ListObject(a.clone())),
        Err(EvalError::NotImplemented("cyclic ordinary value printing"))
    );
    assert_eq!(
        call(
            &mut runtime,
            "identical",
            vec![Sexp::ListObject(a), Sexp::ListObject(b)]
        )?,
        Sexp::Logical(vec![Some(true)])
    );
    assert!(runtime.take_output().is_empty());
    Ok(())
}
