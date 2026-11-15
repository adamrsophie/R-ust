use r_ust_eval::Frontend;
use r_ust_sexp::{AssignmentOperator, ListElement, Sexp};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn expected(frontend: &mut Frontend, source: &str, values: &[bool]) -> Result {
    let actual = frontend.eval_source(source)?;
    assert_eq!(
        actual.last(),
        Some(&Sexp::Logical(values.iter().copied().map(Some).collect())),
        "{source}"
    );
    Ok(())
}

#[test]
fn ordered_class_names_identity_matches_pinned_r_recursively() -> Result {
    let mut frontend = Frontend::default();
    frontend
        .eval_source("x<-list(1L);class(x)<-'foo';names(x)<-'a';y<-list(a=1L);class(y)<-'foo'")?;
    expected(
        &mut frontend,
        "c(identical(x,y),identical(x,y,attrib.as.set=FALSE),identical(y,x,attrib.as.set=FALSE))",
        &[true, false, false],
    )?;
    frontend.eval_source("f<-function()1L;attr(f,'payload')<-x;g<-f;attr(g,'payload')<-y")?;
    expected(
        &mut frontend,
        "c(identical(f,g),identical(f,g,attrib.as.set=FALSE),identical(g,f,attrib.as.set=FALSE))",
        &[true, false, false],
    )?;
    frontend.eval_source("u<-list(1L);class(u)<-'foo';v<-u;names(v)<-NULL")?;
    expected(&mut frontend, "identical(u,v,attrib.as.set=FALSE)", &[true])?;
    Ok(())
}

#[test]
fn removing_names_makes_class_order_irrelevant() -> Result {
    let mut frontend = Frontend::default();
    frontend
        .eval_source("x<-list(a=1L);class(x)<-'foo';names(x)<-NULL;y<-list(1L);class(y)<-'foo'")?;
    expected(
        &mut frontend,
        "c(identical(x,y),identical(x,y,attrib.as.set=FALSE),identical(y,x,attrib.as.set=FALSE))",
        &[true, true, true],
    )?;
    Ok(())
}

fn classed(class_first: bool, names: bool) -> Sexp {
    Sexp::ClassedList {
        elements: vec![ListElement {
            name: names.then(|| "a".into()),
            value: Sexp::Integer(vec![Some(1)]),
        }],
        class: Box::new(Sexp::String(vec![Some("foo".into())])),
        class_first,
    }
}

#[test]
fn sdk_class_first_matters_only_when_two_attributes_exist() -> Result {
    for names in [false, true] {
        let mut frontend = Frontend::default();
        for (name, flag) in [("left", false), ("right", true)] {
            frontend.runtime_mut().eval(&Sexp::Assignment {
                operator: AssignmentOperator::Left,
                target: name.into(),
                value: Box::new(classed(flag, names)),
            })?;
        }
        expected(&mut frontend, "identical(left,right)", &[true])?;
        expected(&mut frontend, "c(identical(left,right,attrib.as.set=FALSE),identical(right,left,attrib.as.set=FALSE))", &[!names, !names])?;
    }
    Ok(())
}
