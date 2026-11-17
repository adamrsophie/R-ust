use super::*;

#[test]
fn ordinary_named_dimnames_preserve_explicit_all_na_names() -> Result {
    let raw = Sexp::list_with_metadata(vec![slot(labels()), slot(Sexp::Nil)], true, None);
    for managed in [false, true] {
        let mut f = Frontend::default();
        bind(f.runtime_mut(), "x", array())?;
        bind(f.runtime_mut(), "d", variant(&raw, managed)?)?;
        truth(&mut f, "attr(x,'dimnames')<-d;identical(attr(x,'dimnames'),d) && identical(names(attr(x,'dimnames')),c(NA_character_,NA_character_))")?;
    }
    Ok(())
}

#[test]
fn managed_dimnames_keep_null_children_metadata_and_child_ids() -> Result {
    let owner = Environment::global();
    let raw = Sexp::AttributedAggregate {
        values: AggregateVector::Vector(vec![labels(), Sexp::Nil]),
        attributes: vec![
            attr("names", Sexp::String(vec![None, None])),
            attr("note", Sexp::Environment(owner.clone())),
            attr("class", string("labels")),
        ],
    };
    for managed_object in [false, true] {
        for managed_rhs in [false, true] {
            let rhs = variant(&raw, managed_rhs)?;
            let before = rhs.snapshot()?;
            let mut f = Frontend::default();
            bind(f.runtime_mut(), "x", variant(&array(), managed_object)?)?;
            bind(f.runtime_mut(), "d", rhs.clone())?;
            truth(&mut f, "attr(x,'dimnames')<-d;identical(attr(x,'dimnames'),d) && identical(names(attr(x,'dimnames')),c(NA_character_,NA_character_))")?;
            let result = attribute(&get(&f, "x")?, "dimnames")?;
            let (children, attrs) = aggregate(&result)?;
            assert_eq!(children[1].snapshot()?, Sexp::Nil);
            assert_eq!(rhs.snapshot()?, before);
            if managed_rhs {
                let (original_children, original_attrs) = aggregate(&rhs)?;
                assert_eq!(children, original_children);
                assert_eq!(attrs, original_attrs);
                if let (Sexp::Managed(old), Sexp::Managed(new)) = (&rhs, &result) {
                    assert!(!old.same_object(new));
                }
            }
        }
    }
    Ok(())
}

#[test]
fn dimnames_container_and_component_metadata_survive_without_resizing() -> Result {
    let nested_names = Sexp::AttributedCharacter {
        values: vec![Some("rows".into()), Some("columns".into())],
        attributes: vec![attr("marker", Sexp::Integer(vec![Some(11)]))],
    };
    let component = Sexp::AttributedCharacter {
        values: vec![Some("a".into()), Some("b".into())],
        attributes: vec![attr("component", Sexp::Integer(vec![Some(13)]))],
    };
    let raw = Sexp::AttributedAggregate {
        values: AggregateVector::Vector(vec![component, Sexp::Nil]),
        attributes: vec![
            attr("names", nested_names),
            attr("note", Sexp::Integer(vec![Some(7)])),
            attr("class", string("labels")),
        ],
    };
    for managed in [false, true] {
        let rhs = variant(&raw, managed)?;
        let mut f = Frontend::default();
        bind(f.runtime_mut(), "x", array())?;
        bind(f.runtime_mut(), "d", rhs.clone())?;
        truth(&mut f, "attr(x,'dimnames')<-d;identical(attr(x,'dimnames'),d) && identical(attr(names(attr(x,'dimnames')),'marker'),11L) && identical(attr(attr(x,'dimnames')[[1]],'component'),13L)")?;
        let result = attribute(&get(&f, "x")?, "dimnames")?;
        assert_eq!(result.snapshot()?, rhs.snapshot()?);
        if managed {
            assert_eq!(aggregate(&result)?, aggregate(&rhs)?);
        }
    }
    Ok(())
}

#[test]
fn dimnames_padding_keeps_only_resized_names_and_surviving_children() -> Result {
    let raw = Sexp::AttributedAggregate {
        values: AggregateVector::Vector(vec![labels()]),
        attributes: vec![
            attr(
                "names",
                Sexp::AttributedCharacter {
                    values: vec![None],
                    attributes: vec![attr("nested", Sexp::Integer(vec![Some(11)]))],
                },
            ),
            attr("note", Sexp::Integer(vec![Some(7)])),
            attr("class", string("labels")),
        ],
    };
    for managed in [false, true] {
        let rhs = variant(&raw, managed)?;
        let before = rhs.snapshot()?;
        let mut f = Frontend::default();
        bind(f.runtime_mut(), "x", array())?;
        bind(f.runtime_mut(), "d", rhs.clone())?;
        f.eval_source("attr(x,'dimnames')<-d")?;
        let (children, attrs) = aggregate(&attribute(&get(&f, "x")?, "dimnames")?)?;
        assert_eq!(children.len(), 2);
        assert_eq!(children[1].snapshot()?, Sexp::Nil);
        assert_eq!(attrs.len(), 1);
        assert_eq!(attrs[0].name.as_deref(), Some("names"));
        assert_eq!(
            attrs[0].value.snapshot()?,
            Sexp::String(vec![None, Some(String::new())])
        );
        assert_eq!(rhs.snapshot()?, before);
        if managed {
            assert_eq!(children[0], aggregate(&rhs)?.0[0]);
        }
    }
    Ok(())
}

#[test]
fn dimnames_pairlist_tags_are_not_installed_as_list_names() -> Result {
    let mut f = Frontend::default();
    truth(&mut f, "x<-structure(1:4,dim=c(2L,2L));attr(x,'dimnames')<-pairlist(rows=c('a','b'),columns=c('c','d'));identical(attr(x,'dimnames'),list(c('a','b'),c('c','d'))) && is.null(names(attr(x,'dimnames')))")
}
