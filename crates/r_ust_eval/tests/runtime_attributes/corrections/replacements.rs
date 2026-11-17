use super::*;

#[test]
fn raw_and_managed_attribute_lists_keep_order_special_names_and_child_ids() -> Result {
    let raw = Sexp::AttributedAggregate {
        values: AggregateVector::Vector(vec![Sexp::Integer(vec![Some(7)]), string("custom")]),
        attributes: vec![attr(
            "names",
            Sexp::String(vec![Some("note".into()), Some("class".into())]),
        )],
    };
    for managed_object in [false, true] {
        for managed_rhs in [false, true] {
            let object = variant(&Sexp::Integer(vec![Some(1), Some(2)]), managed_object)?;
            let replacement = variant(&raw, managed_rhs)?;
            let before = replacement.snapshot()?;
            let mut f = Frontend::default();
            bind(f.runtime_mut(), "x", object.clone())?;
            bind(f.runtime_mut(), "a", replacement.clone())?;
            truth(&mut f, "original<-x;attributes(x)<-a;identical(attributes(x),list(note=7L,class='custom')) && is.object(x) && is.null(attributes(original))")?;
            let changed = get(&f, "x")?;
            assert_eq!(replacement.snapshot()?, before);
            assert_eq!(metadata(&object)?, Vec::<ListElement>::new());
            if managed_rhs {
                let (children, _) = aggregate(&replacement)?;
                assert_eq!(attribute(&changed, "note")?, children[0]);
                assert_eq!(attribute(&changed, "class")?, children[1]);
            }
            if managed_object {
                let Sexp::Managed(changed) = changed else {
                    panic!("Managed target lost its graph handle");
                };
                assert!(changed.view()?.header.is_object());
            }
        }
    }
    Ok(())
}

#[test]
fn managed_attribute_list_sets_dimensions_before_dimnames() -> Result {
    let raw = Sexp::AttributedAggregate {
        values: AggregateVector::Vector(vec![
            Sexp::List(vec![slot(labels()), slot(labels())]),
            Sexp::Integer(vec![Some(2), Some(2)]),
            Sexp::Integer(vec![Some(7)]),
        ]),
        attributes: vec![attr(
            "names",
            Sexp::String(vec![
                Some("dimnames".into()),
                Some("dim".into()),
                Some("note".into()),
            ]),
        )],
    };
    for managed in [false, true] {
        let mut f = Frontend::default();
        bind(f.runtime_mut(), "a", variant(&raw, managed)?)?;
        truth(&mut f, "x<-1:4;attributes(x)<-a;identical(names(attributes(x)),c('dim','dimnames','note')) && identical(attr(x,'dim'),c(2L,2L)) && identical(attr(x,'dimnames'),list(c('a','b'),c('a','b'))) && identical(attr(x,'note'),7L)")?;
    }
    Ok(())
}

#[test]
fn managed_factor_names_and_dimnames_use_levels() -> Result {
    let factor = Sexp::AttributedAtomic {
        values: AtomicVector::Integer(vec![Some(1), Some(2), None]),
        attributes: vec![
            attr(
                "levels",
                Sexp::String(vec![Some("A".into()), Some("B".into())]),
            ),
            attr("class", string("factor")),
        ],
    };
    for managed in [false, true] {
        let rhs = variant(&factor, managed)?;
        let before = rhs.snapshot()?;
        let mut f = Frontend::default();
        bind(f.runtime_mut(), "v", rhs.clone())?;
        bind(
            f.runtime_mut(),
            "d",
            variant(&Sexp::List(vec![slot(factor.clone())]), managed)?,
        )?;
        truth(
            &mut f,
            "x<-1:3;names(x)<-v;identical(names(x),c('A','B',NA_character_))",
        )?;
        truth(&mut f, "x<-structure(1:3,dim=3L);attr(x,'dimnames')<-d;identical(attr(x,'dimnames'),list(c('A','B',NA_character_)))")?;
        assert_eq!(rhs.snapshot()?, before);
    }
    Ok(())
}
