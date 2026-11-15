use super::support;

#[test]
fn default_numeric() {
    support::check(
        "default_numeric",
        r###"c(identical(0,-0),identical(NA_real_,NA_real_),identical(NaN,NaN),identical(NA_real_,NaN))"###,
    );
}

#[test]
fn explicit_defaults() {
    support::check(
        "explicit_defaults",
        r###"identical(1L,1L,TRUE,TRUE,TRUE,TRUE,FALSE,TRUE,FALSE)"###,
    );
}

#[test]
fn partial_option() {
    support::check("partial_option", r###"identical(1L,1L,num=TRUE)"###);
}

#[test]
fn logical_first() {
    support::check(
        "logical_first",
        r###"identical(1L,1L,num.eq=c(TRUE,FALSE))"###,
    );
}

#[test]
fn numeric_flag() {
    support::check(
        "numeric_flag",
        r###"identical(1L,1L,num.eq=2L,single.NA=-1,ignore.environment=0,extptr.as.ref=0L)"###,
    );
}

#[test]
fn text_flag() {
    support::check(
        "text_flag",
        r###"identical(1L,1L,num.eq='true',ignore.environment='FALSE')"###,
    );
}

#[test]
fn attribute_order() {
    support::check(
        "attribute_order",
        r###"a<-structure('x',first=1L,second=2L);b<-structure('x',second=2L,first=1L);c(identical(a,b),identical(a,b,attrib.as.set=FALSE))"###,
    );
}

#[test]
fn srcref_option() {
    support::check(
        "srcref_option",
        r###"a<-function(x)x;b<-a;attr(a,'srcref')<-1L;attr(b,'srcref')<-2L;c(identical(a,b),identical(a,b,ignore.srcref=FALSE))"###,
    );
}

#[test]
fn internal_five() {
    support::check(
        "internal_five",
        r###".Internal(identical(1L,1L,TRUE,TRUE,TRUE))"###,
    );
}

#[test]
fn internal_six() {
    support::check(
        "internal_six",
        r###".Internal(identical(1L,1L,TRUE,TRUE,TRUE,TRUE))"###,
    );
}

#[test]
fn internal_seven() {
    support::check(
        "internal_seven",
        r###".Internal(identical(1L,1L,TRUE,TRUE,TRUE,TRUE,FALSE))"###,
    );
}

#[test]
fn internal_eight() {
    support::check(
        "internal_eight",
        r###".Internal(identical(1L,1L,TRUE,TRUE,TRUE,TRUE,FALSE,TRUE))"###,
    );
}

#[test]
fn internal_nine() {
    support::check(
        "internal_nine",
        r###".Internal(identical(1L,1L,TRUE,TRUE,TRUE,TRUE,FALSE,TRUE,FALSE))"###,
    );
}

#[test]
fn internal_extra() {
    support::check(
        "internal_extra",
        r###".Internal(identical(1L,1L,TRUE,TRUE,TRUE,TRUE,FALSE,TRUE,FALSE,{cat('EXTRA|');NA}))"###,
    );
}

#[test]
fn internal_names_positional() {
    support::check(
        "internal_names_positional",
        r###".Internal(identical(y=1L,x=2L,num.eq=TRUE,single.NA=TRUE,attrib.as.set=TRUE))"###,
    );
}
