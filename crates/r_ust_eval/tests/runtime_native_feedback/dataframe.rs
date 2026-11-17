use super::*;

#[test]
fn renamed_legacy_dataframe_keeps_indexing_assignment_and_original_alias() -> Result {
    let mut f = Frontend::default();
    truth(&mut f, "df<-data.frame(x=1:3,y=c('a','b','c'));old<-df;names(df)<-c('a','b');df[1,'a']<-99L;identical(df$a,c(99L,2L,3L)) && identical(names(old),c('x','y')) && identical(old$x,1:3) && is.data.frame(df)")?;
    assert!(matches!(get(&f, "df")?, Sexp::DataFrame { nrow: 3, .. }));
    Ok(())
}

#[test]
fn legacy_dataframe_renaming_preserves_nan_bits_and_managed_column_identity() -> Result {
    let mut f = Frontend::default();
    let owner = global_environment(&mut f)?;
    let child = ValueObject::import(
        &owner,
        &Sexp::Real(vec![f64::NAN, r_ust_sexp::NA_REAL, f64::INFINITY]),
        ValueImport::Evaluated,
    )?;
    let raw = Sexp::DataFrame {
        columns: vec![attr("x", child.transport()?)],
        nrow: 3,
    };
    bind(&mut f, "df", raw)?;
    f.eval_source("old<-df;names(df)<-'a'")?;
    let Sexp::DataFrame { columns, nrow } = get(&f, "df")? else {
        panic!("losslessly representable data frame")
    };
    assert_eq!(nrow, 3);
    assert_eq!(columns[0].name.as_deref(), Some("a"));
    let Sexp::Managed(retained) = &columns[0].value else {
        panic!("managed column retained")
    };
    assert!(retained.same_object(&child));
    let Sexp::DataFrame { columns: old, .. } = get(&f, "old")? else {
        panic!("original data frame")
    };
    assert_eq!(old[0].name.as_deref(), Some("x"));
    drop(old);
    drop(child);
    drop(owner);
    drop(f);
    let Sexp::Real(values) = retained.snapshot()? else {
        panic!("real column")
    };
    assert!(values[0].is_nan() && !r_ust_sexp::is_na_real(values[0]));
    assert!(r_ust_sexp::is_na_real(values[1]));
    assert_eq!(values[2], f64::INFINITY);
    Ok(())
}

#[test]
fn unrepresentable_dataframe_metadata_stays_typed_and_exact() -> Result {
    for (update, check) in [
        ("attr(df,'note')<-7L", "identical(attr(df,'note'),7L)"),
        (
            "attr(df,'row.names')<-c('r1','r2','r3')",
            "identical(attr(df,'row.names'),c('r1','r2','r3'))",
        ),
        (
            "attr(df,'class')<-c('custom','data.frame')",
            "identical(attr(df,'class'),c('custom','data.frame'))",
        ),
    ] {
        let mut f = Frontend::default();
        truth(&mut f, &format!("df<-data.frame(x=1:3);{update};names(df)<-'a';{check} && identical(df[[1]],1:3) && identical(names(df),'a')"))?;
        assert!(matches!(get(&f, "df")?, Sexp::AttributedAggregate { .. }));
    }
    Ok(())
}

#[test]
fn dataframe_managed_names_metadata_is_not_flattened_into_legacy_fields() -> Result {
    let mut f = Frontend::default();
    let owner = global_environment(&mut f)?;
    let names = managed(
        &owner,
        &Sexp::AttributedCharacter {
            values: vec![Some("a".into())],
            attributes: vec![attr("note", Sexp::Integer(vec![Some(7)]))],
        },
    )?;
    bind(&mut f, "v", names.clone())?;
    truth(&mut f, "df<-data.frame(x=1:3);attr(df,'names')<-v;identical(names(df),v) && identical(df[[1]],1:3)")?;
    let Sexp::AttributedAggregate { attributes, .. } = get(&f, "df")? else {
        panic!("names metadata requires general descriptor")
    };
    assert_eq!(attributes[0], attr("names", names));
    Ok(())
}

#[test]
fn dataframe_reordered_attributes_do_not_canonicalize_silently() -> Result {
    let mut f = Frontend::default();
    f.eval_source("df<-data.frame(x=1:3)")?;
    let Some(Sexp::AttributedAggregate {
        values,
        mut attributes,
    }) = get(&f, "df")?.attribute_description()?
    else {
        panic!("data frame descriptor")
    };
    attributes.reverse();
    bind(
        &mut f,
        "df",
        Sexp::AttributedAggregate { values, attributes },
    )?;
    truth(&mut f, "before<-names(attributes(df));names(df)<-'a';identical(names(attributes(df)),before) && identical(df[[1]],1:3)")?;
    assert!(matches!(get(&f, "df")?, Sexp::AttributedAggregate { .. }));
    Ok(())
}
