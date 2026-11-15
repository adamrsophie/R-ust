use super::support;

#[test]
fn invalid_flag() {
    support::check("invalid_flag", r###"identical(1L,1L,num.eq=NA)"###);
}

#[test]
fn empty_flag() {
    support::check("empty_flag", r###"identical(1L,1L,num.eq=logical())"###);
}

#[test]
fn invalid_later_flag() {
    support::check(
        "invalid_later_flag",
        r###"identical(1L,1L,num.eq=FALSE,single.NA=NA)"###,
    );
}

#[test]
fn all_operands_before_validation() {
    support::check(
        "all_operands_before_validation",
        r###"identical({cat('X|');1L},{cat('Y|');1L},num.eq={cat('N|');NA},single.NA={cat('S|');TRUE})"###,
    );
}

#[test]
fn internal_zero() {
    support::check(
        "internal_zero",
        r###"f<-function().Internal(identical());f()"###,
    );
}

#[test]
fn internal_one() {
    support::check(
        "internal_one",
        r###"f<-function().Internal(identical(1L));f()"###,
    );
}

#[test]
fn internal_four() {
    support::check(
        "internal_four",
        r###"f<-function().Internal(identical(1L,1L,TRUE,TRUE));f()"###,
    );
}

#[test]
fn unknown_argument() {
    support::check("unknown_argument", r###"identical(1L,1L,unknown=TRUE)"###);
}

#[test]
fn ambiguous_argument() {
    support::check("ambiguous_argument", r###"identical(1L,1L,ignore=TRUE)"###);
}

#[test]
fn duplicate_argument() {
    support::check("duplicate_argument", r###"identical(x=1L,x=1L,y=1L)"###);
}

#[test]
fn primitive_lookup() {
    support::check("primitive_lookup", r###".Primitive('identical')"###);
}

#[test]
fn missing_x() {
    support::check("missing_x", r###"identical()"###);
}

#[test]
fn missing_y() {
    support::check("missing_y", r###"identical(1L)"###);
}

#[test]
fn promised_missing() {
    support::check(
        "promised_missing",
        r###"f<-function(x)identical(x,1L);f()"###,
    );
}
