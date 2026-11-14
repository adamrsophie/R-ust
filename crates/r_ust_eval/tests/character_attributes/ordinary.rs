use super::*;

pub(super) fn run() -> Result {
    frozen::ordinary_observer_boundary()?;
    let mut f = Frontend::default();
    f.eval_source("x<-structure(c('b',NA_character_),previous=c('a','b'),custom=list(1L,'z'))")?;
    for source in [
        "identical(names(attributes(x)),c('previous','custom'))",
        "!is.object(x) && !is.vector(x) && is.character(x) && is.atomic(x)",
        "identical(attr(x,'previous'),c('a','b'))",
        "identical(attr(x,'custom'),list(1L,'z'))",
        "!identical(x,c('b',NA_character_))",
        "identical(x,structure(c('b',NA_character_),custom=list(1L,'z'),previous=c('a','b')))",
        "attr(x,'previous')<-'changed'; identical(names(attributes(x)),c('previous','custom'))",
        "attr(x,'previous')<-NULL; identical(names(attributes(x)),'custom')",
        "attr(x,'previous')<-'again'; identical(names(attributes(x)),c('custom','previous'))",
        "attributes(x)<-list(custom=1L,previous='x',custom=2L); identical(attributes(x),list(custom=2L,previous='x'))",
        "attributes(x)<-list(custom=1L,custom=NULL); is.null(attributes(x)) && is.vector(x)",
        "x<-structure(character(),custom=list(NA_character_)); length(x)==0L && identical(attr(x,'custom'),list(NA_character_))",
    ] { truth(&mut f, source)?; }
    f.eval_source("attributes(x)<-NULL")?;
    assert_eq!(f.runtime().get("x")?, Some(strings(&[])));
    malformed(&mut f)?;
    for name in [
        "names",
        "class",
        "dim",
        "dimnames",
        "levels",
        "tsp",
        "row.names",
        "comment",
    ] {
        f.eval_source("x<-'keep'")?;
        assert!(matches!(
            f.eval_source(&format!("attr(x,'{name}')<-1L")),
            Err(EvalError::NotImplemented(_))
        ));
        assert_eq!(f.runtime().get("x")?, Some(strings(&[Some("keep")])));
    }
    Ok(())
}

fn malformed(frontend: &mut Frontend) -> Result {
    let scalar = strings(&[Some("payload")]);
    let mut states = vec![
        vec![ListElement {
            name: None,
            value: scalar.clone(),
        }],
        vec![attribute("", scalar.clone())],
        vec![
            attribute("custom", scalar.clone()),
            attribute("custom", scalar.clone()),
        ],
        vec![attribute("custom", Sexp::Nil)],
    ];
    for name in [
        "names",
        "class",
        "dim",
        "dimnames",
        "levels",
        "tsp",
        "row.names",
        "comment",
    ] {
        states.push(vec![attribute(name, scalar.clone())]);
    }
    bind(
        frontend.runtime_mut(),
        "retained",
        strings(&[Some("unchanged")]),
    )?;
    for attrs in states {
        let value = attributed(&[Some("bad")], attrs);
        assert!(format_sexp(&value).is_err(), "formatter accepted {value:?}");
        for invalid in [value.clone(), Sexp::List(vec![attribute("nested", value)])] {
            assert!(bind(frontend.runtime_mut(), "retained", invalid).is_err());
            assert_eq!(
                frontend.runtime().get("retained")?,
                Some(strings(&[Some("unchanged")]))
            );
        }
    }
    Ok(())
}
