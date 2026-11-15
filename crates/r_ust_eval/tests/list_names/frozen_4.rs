use super::*;

#[test]
fn empty_presence_identity() -> Result {
    run(
        r####"x<-list();y<-list();names(y)<-character();identical(x,y)"####,
        Sexp::Logical(vec![Some(false)]),
        "",
    )
}

#[test]
fn all_na_vs_absent_identity() -> Result {
    run(
        r####"x<-list(1L);y<-x;names(y)<-NA_character_;identical(x,y)"####,
        Sexp::Logical(vec![Some(false)]),
        "",
    )
}
