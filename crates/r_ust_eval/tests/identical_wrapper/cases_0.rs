use super::support;

#[test]
fn wrapper_type() {
    support::check("wrapper_type", r###"typeof(identical)"###);
}

#[test]
fn wrapper_formals() {
    support::check("wrapper_formals", r###"names(formals(identical))"###);
}

#[test]
fn wrapper_body() {
    support::check(
        "wrapper_body",
        r###"identical(body(identical),quote(.Internal(identical(x,y,num.eq,single.NA,attrib.as.set,ignore.bytecode,ignore.environment,ignore.srcref,extptr.as.ref))))"###,
    );
}

#[test]
fn wrapper_environment() {
    support::check(
        "wrapper_environment",
        r###"environmentName(environment(identical))"###,
    );
}

#[test]
fn direct_parent() {
    support::check(
        "direct_parent",
        r###"f<-function()identical(as.environment(-1L),parent.frame());f()"###,
    );
}

#[test]
fn delayed_parent() {
    support::check(
        "delayed_parent",
        r###"f<-function(){e<-as.environment(-1L);identical(e,parent.frame())};f()"###,
    );
}

#[test]
fn alias_parent() {
    support::check(
        "alias_parent",
        r###"i<-identical;f<-function()i(as.environment(-1L),parent.frame());f()"###,
    );
}

#[test]
fn numeric_global() {
    support::check(
        "numeric_global",
        r###"f<-function()identical(as.environment(0L),globalenv());f()"###,
    );
}

#[test]
fn same_environment() {
    support::check("same_environment", r###"e<-new.env();identical(e,e)"###);
}

#[test]
fn distinct_environments() {
    support::check(
        "distinct_environments",
        r###"identical(new.env(),new.env())"###,
    );
}

#[test]
fn argument_order() {
    support::check(
        "argument_order",
        r###"identical(y={cat('Y|');1L},x={cat('X|');1L},ignore.srcref={cat('S|');TRUE},num.eq={cat('N|');TRUE})"###,
    );
}

#[test]
fn promise_once() {
    support::check(
        "promise_once",
        r###"f<-function(z)identical(z,z);f({cat('ONCE|');1L})"###,
    );
}

#[test]
fn visible_result() {
    support::check(
        "visible_result",
        r###"identical(invisible(1L),invisible(1L))"###,
    );
}

#[test]
fn integer_equal() {
    support::check("integer_equal", r###"identical(1L,1L)"###);
}

#[test]
fn integer_double() {
    support::check("integer_double", r###"identical(1L,1)"###);
}
