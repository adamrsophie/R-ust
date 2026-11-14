use super::*;
use r_ust_sexp::PairListObject;

#[test]
fn ordinary_pairlist_surfaces() -> Result {
    let owner = Environment::global();
    let Sexp::List(elements) = payload() else {
        panic!("list payload")
    };
    let tail = PairListObject::cons(&owner, &elements[1], None)?;
    let head = PairListObject::cons(&owner, &elements[0], Some(&tail))?;
    for source in [
        "typeof(x)",
        "mode(x)",
        "class(x)",
        "length(x)",
        "names(x)",
        "attributes(x)",
        "is.list(x)",
        "is.atomic(x)",
        "is.vector(x)",
        "is.na(x)",
        "anyNA(x)",
        "as.list(x)",
        "as.pairlist(x)",
        "x[1L]",
        "x[[1L]]",
        "x$a",
        "print(x)",
        "names(x)<-c('c','d');names(x)",
    ] {
        let mut inline = Frontend::default();
        let mut shared = Frontend::default();
        bind(inline.runtime_mut(), Sexp::pairlist(elements.clone()))?;
        bind(shared.runtime_mut(), Sexp::PairListObject(head.clone()))?;
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
        assert!(head.cdr()?.is_some_and(|value| value.same_object(&tail)));
        assert_eq!(head.car()?, elements[0]);
    }
    Ok(())
}

#[test]
fn pairlist_public_replacement_preserves_alias_and_nested_identity() -> Result {
    let owner = Environment::global();
    let nested = ListObject::from_value(&owner, &payload())?;
    let tail = PairListObject::cons(
        &owner,
        &ListElement {
            name: None,
            value: Sexp::ListObject(nested.clone()),
        },
        None,
    )?;
    let head = PairListObject::cons(
        &owner,
        &ListElement {
            name: Some("a".into()),
            value: Sexp::Integer(vec![Some(1)]),
        },
        Some(&tail),
    )?;
    for source in [
        "x[[1L]]<-9L",
        "x$a<-9L",
        "x$c<-9L",
        "x[[4L]]<-9L",
        "names(x)<-c('c','d')",
        "names(x)<-NA_character_",
        "attributes(x)<-NULL",
    ] {
        let mut frontend = Frontend::default();
        bind(frontend.runtime_mut(), Sexp::PairListObject(head.clone()))?;
        frontend.eval_source(source)?;
        let Some(Sexp::PairListObject(copy)) = frontend.runtime().get("x")? else {
            panic!("{source}: cons storage lost")
        };
        assert!(!copy.same_object(&head), "{source}");
        let expected_names = match source {
            "names(x)<-NA_character_" | "attributes(x)<-NULL" => Sexp::Nil,
            "names(x)<-c('c','d')" => Sexp::String(vec![Some("c".into()), Some("d".into())]),
            "x$c<-9L" => Sexp::String(vec![
                Some("a".into()),
                Some(String::new()),
                Some("c".into()),
            ]),
            "x[[4L]]<-9L" => Sexp::String(vec![
                Some("a".into()),
                Some(String::new()),
                Some(String::new()),
                Some(String::new()),
            ]),
            _ => Sexp::String(vec![Some("a".into()), Some(String::new())]),
        };
        assert_eq!(
            frontend.eval_source("names(x)")?,
            vec![expected_names],
            "{source}"
        );
        let copied_tail = copy.cdr()?.ok_or("tail")?;
        assert!(
            !copied_tail.same_object(&tail),
            "{source}: spine was not copied"
        );
        let Sexp::ListObject(retained) = copied_tail.car()?.value else {
            panic!("{source}: nested list")
        };
        assert!(retained.same_object(&nested), "{source}");
        assert!(head.cdr()?.is_some_and(|value| value.same_object(&tail)));
        assert_eq!(head.car()?.value, Sexp::Integer(vec![Some(1)]));
    }
    for source in ["x[[1L]]<-NULL", "x$a<-NULL"] {
        let mut frontend = Frontend::default();
        bind(frontend.runtime_mut(), Sexp::PairListObject(head.clone()))?;
        frontend.eval_source(source)?;
        let Some(Sexp::PairListObject(copy)) = frontend.runtime().get("x")? else {
            panic!("{source}: cons")
        };
        assert!(copy.cdr()?.is_none());
        let Sexp::ListObject(retained) = copy.car()?.value else {
            panic!("nested list")
        };
        assert!(retained.same_object(&nested));
    }
    let mut frontend = Frontend::default();
    bind(frontend.runtime_mut(), Sexp::PairListObject(head.clone()))?;
    frontend.eval_source("x[1L]<-list(9L)")?;
    assert!(matches!(frontend.runtime().get("x")?, Some(Sexp::List(_))));
    assert_eq!(head.car()?.value, Sexp::Integer(vec![Some(1)]));
    Ok(())
}
