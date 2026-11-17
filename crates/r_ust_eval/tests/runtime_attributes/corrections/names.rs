use super::*;

#[test]
fn managed_plain_names_keep_the_assigned_character_child() -> Result {
    let names = variant(&labels(), true)?;
    let mut f = Frontend::default();
    bind(f.runtime_mut(), "v", names.clone())?;
    truth(
        &mut f,
        "x<-list(1L,2L);attr(x,'names')<-v;identical(names(x),c('a','b'))",
    )?;
    assert_eq!(attribute(&get(&f, "x")?, "names")?, names);
    Ok(())
}

#[test]
fn names_vectors_keep_nested_metadata_and_managed_identity() -> Result {
    let names = Sexp::AttributedCharacter {
        values: vec![Some("a".into()), Some("b".into())],
        attributes: vec![attr("note", Sexp::Integer(vec![Some(7)]))],
    };
    for raw in [
        Sexp::List(vec![
            slot(Sexp::Integer(vec![Some(1)])),
            slot(Sexp::Integer(vec![Some(2)])),
        ]),
        Sexp::Expression {
            elements: vec![
                slot(Sexp::Symbol("a".into())),
                slot(Sexp::Symbol("b".into())),
            ],
            names_present: false,
        },
    ] {
        for managed_object in [false, true] {
            for managed_names in [false, true] {
                let object = variant(&raw, managed_object)?;
                let names = variant(&names, managed_names)?;
                let mut f = Frontend::default();
                bind(f.runtime_mut(), "x", object.clone())?;
                bind(f.runtime_mut(), "v", names.clone())?;
                truth(&mut f, "attr(x,'names')<-v;identical(names(x),v) && identical(attributes(names(x)),list(note=7L))")?;
                let changed = get(&f, "x")?;
                let assigned = attribute(&changed, "names")?;
                assert_eq!(assigned.snapshot()?, names.snapshot()?);
                if managed_names {
                    assert_eq!(assigned, names);
                }
                assert_eq!(changed.sexptype(), object.sexptype());
                assert!(metadata(&object)?.is_empty());
                payload_shared(&object, &changed, 2)?;
            }
        }
    }
    Ok(())
}

#[test]
fn padding_names_drops_other_metadata_and_resizes_nested_names() -> Result {
    for nested_names in [false, true] {
        let mut attrs = vec![attr("note", Sexp::Integer(vec![Some(7)]))];
        if nested_names {
            attrs.insert(0, attr("names", string("first")));
        }
        let names = Sexp::AttributedCharacter {
            values: vec![Some("a".into())],
            attributes: attrs,
        };
        for managed in [false, true] {
            let rhs = variant(&names, managed)?;
            let before = rhs.snapshot()?;
            let mut f = Frontend::default();
            bind(f.runtime_mut(), "v", rhs.clone())?;
            f.eval_source("x<-1:3;attr(x,'names')<-v")?;
            let assigned = attribute(&get(&f, "x")?, "names")?;
            let expected = if nested_names {
                Sexp::AttributedCharacter {
                    values: vec![Some("a".into()), None, None],
                    attributes: vec![attr(
                        "names",
                        Sexp::String(vec![
                            Some("first".into()),
                            Some(String::new()),
                            Some(String::new()),
                        ]),
                    )],
                }
            } else {
                Sexp::String(vec![Some("a".into()), None, None])
            };
            assert_eq!(assigned.snapshot()?, expected);
            assert_eq!(rhs.snapshot()?, before);
        }
    }
    Ok(())
}
