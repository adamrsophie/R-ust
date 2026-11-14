use super::*;

#[test]
fn class_rhs_metadata_matches_independent_r() -> Result {
    for (source, expected) in [
        ("x<-list(1L);v<-structure('box',custom=1L);class(x)<-v;print(attributes(class(x)));print(attributes(x)[['class']]);cat(identical(class(x),v))", "$custom\n[1] 1\n\n[1] \"box\"\nattr(,\"custom\")\n[1] 1\nTRUE"),
        ("v<-structure('box',custom=1L);x<-structure(list(1L),class=v);print(attributes(class(x)));print(attributes(x)[['class']]);cat(identical(class(x),v))", "$custom\n[1] 1\n\n[1] \"box\"\nattr(,\"custom\")\n[1] 1\nTRUE"),
        ("v<-structure('box',custom=1L);x<-list(1L);attributes(x)<-list(class=v);print(attributes(class(x)));cat(identical(class(x),v))", "$custom\n[1] 1\n\nTRUE"),
        ("v<-structure(c('box','other'),custom=1L);x<-list(1L);class(x)<-v;print(attributes(class(x)));cat(identical(class(x),v))", "$custom\n[1] 1\n\nTRUE"),
        ("v<-structure('list',custom=1L);x<-structure(list(1L),class='box');class(x)<-v;cat(is.object(x),is.null(attributes(x)),sep='|')", "FALSE|TRUE"),
    ] {
        let mut f = Frontend::default();
        f.eval_source(source)?;
        assert_eq!(stdout(&mut f), expected, "{source}");
    }
    Ok(())
}

#[test]
fn full_class_attribute_display_matches_r() -> Result {
    let mut f = Frontend::default();
    f.eval_source("v<-structure(c('box','other'),custom=list(tag=1L));x<-structure(list(a=1L),class=v);print(x);print(x,quote=FALSE);print(list(nested=x))")?;
    assert_eq!(stdout(&mut f), "$a\n[1] 1\n\nattr(,\"class\")\n[1] \"box\"   \"other\"\nattr(,\"class\")attr(,\"custom\")\nattr(,\"class\")attr(,\"custom\")$tag\n[1] 1\n\n$a\n[1] 1\n\nattr(,\"class\")\n[1] box   other\nattr(,\"class\")attr(,\"custom\")\nattr(,\"class\")attr(,\"custom\")$tag\n[1] 1\n\n$nested\n$a\n[1] 1\n\nattr(,\"class\")\n[1] \"box\"   \"other\"\nattr(,\"class\")attr(,\"custom\")\nattr(,\"class\")attr(,\"custom\")$tag\n[1] 1\n\n\n");
    Ok(())
}

#[test]
fn class_metadata_identity_preservation_and_validation() -> Result {
    let mut f = Frontend::default();
    truth(&mut f, "v<-structure('box',custom=list(NaN,1L));x<-structure(list(a=1L,b=2L),class=v);identical(class(x),v) && identical(attributes(x)$class,v)")?;
    for operation in [
        "names(y)<-c('c','d')",
        "y$a<-3L",
        "y$c<-3L",
        "y$a<-NULL",
        "y[['c']]<-3L",
        "y[['a']]<-NULL",
        "y[1L]<-list(9L)",
    ] {
        truth(&mut f, &format!("y<-x;{operation};identical(class(y),v)"))?;
    }
    truth(&mut f, "a<-structure(list(1L),class=structure('box',one=1L,two=2L));b<-structure(list(1L),class=structure('box',two=2L,one=1L));identical(a,b)")?;
    truth(&mut f, "x<-list(1L);class(x)<-structure(character(),custom=1L);!is.object(x) && is.null(attributes(x))")?;
    truth(
        &mut f,
        "x<-structure(list(1L),class=structure(character(),custom=1L));!is.object(x)",
    )?;
    for class in [
        Sexp::Integer(vec![Some(1)]),
        strings(&[]),
        attributed(
            &[Some("box")],
            vec![attribute("names", strings(&[Some("n")]))],
        ),
    ] {
        let malformed = Sexp::ClassedList {
            elements: vec![],
            class: Box::new(class),
            class_first: true,
        };
        f.eval_source("saved<-list(1L);x<-saved")?;
        assert!(bind(f.runtime_mut(), "x", malformed.clone()).is_err());
        assert!(format_sexp(&malformed).is_err());
        truth(&mut f, "identical(x,saved)")?;
    }
    Ok(())
}

#[test]
fn class_payload_survives_origin_and_consumer_runtime_drops() -> Result {
    sdk::run_class()
}
