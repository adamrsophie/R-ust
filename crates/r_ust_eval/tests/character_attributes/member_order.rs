use super::*;

#[test]
fn reconstruction_preserves_r_attribute_order() -> Result {
    for (source, expected) in [
        ("x<-structure(list(1L,2L),class='box',names=c('a','b'));x$a<-3L;cat(names(attributes(x)),sep='|')", "class|names"),
        ("x<-structure(list(1L,2L),class='box',names=c('a','b'));x$c<-3L;cat(names(attributes(x)),sep='|')", "names|class"),
        ("x<-structure(list(1L,2L),class='box',names=c('a','b'));x$a<-NULL;cat(names(attributes(x)),sep='|')", "names|class"),
        ("x<-structure(list(1L,2L),class='box',names=c('a','b'));x[['a']]<-3L;cat(names(attributes(x)),sep='|')", "class|names"),
        ("x<-structure(list(1L,2L),class='box',names=c('a','b'));x[['c']]<-3L;cat(names(attributes(x)),sep='|')", "names|class"),
        ("x<-structure(list(1L,2L),class='box',names=c('a','b'));x[['a']]<-NULL;cat(names(attributes(x)),sep='|')", "names|class"),
        ("x<-structure(list(a=1L,b=2L),class='box');x$a<-3L;cat(names(attributes(x)),sep='|')", "names|class"),
        ("x<-structure(list(a=1L,b=2L),class='box');x$c<-3L;cat(names(attributes(x)),sep='|')", "names|class"),
        ("x<-structure(list(a=1L,b=2L),class='box');x$a<-NULL;cat(names(attributes(x)),sep='|')", "names|class"),
        ("x<-structure(list(a=1L,b=2L),class='box');x[['a']]<-3L;cat(names(attributes(x)),sep='|')", "names|class"),
        ("x<-structure(list(a=1L,b=2L),class='box');x[['c']]<-3L;cat(names(attributes(x)),sep='|')", "names|class"),
        ("x<-structure(list(a=1L,b=2L),class='box');x[['a']]<-NULL;cat(names(attributes(x)),sep='|')", "names|class"),
        ("x<-structure(list(1L,2L),class='box');x$c<-3L;cat(names(attributes(x)),sep='|')", "names|class"),
        ("x<-structure(list(1L,2L),class='box');x[['c']]<-3L;cat(names(attributes(x)),sep='|')", "class|names"),
    ] {
        let mut frontend = Frontend::default();
        frontend.eval_source(source)?;
        assert_eq!(stdout(&mut frontend),expected,"{source}");
    }
    for operation in ["x$a<-NULL", "x[['a']]<-NULL"] {
        let mut frontend = Frontend::default();
        truth(&mut frontend,&format!("x<-structure(list(a=1L,b=2L),class='box');{operation};length(x)==1L && identical(names(x),'b') && identical(x$b,2L)"))?;
    }
    for operation in ["x$c<-3L", "x[['c']]<-3L"] {
        let mut frontend = Frontend::default();
        truth(&mut frontend,&format!("x<-structure(list(1L,2L),class='box');{operation};identical(names(x),c('','','c'))"))?;
        truth(&mut frontend,&format!("x<-structure(list(1L,2L),class='box',names=c(NA_character_,'b'));{operation};identical(names(x),c(NA_character_,'b','c'))"))?;
    }
    Ok(())
}
