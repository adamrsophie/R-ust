use super::{check, Row};

#[test]
fn dep_call_invalid_order() {
    check(Row {
        id: "dep_call_invalid_order",
        source: r####"n<-0L;tryCatch(call(1L,{n<-n+1L;n}),error=function(e)NULL);as.character(n)"####,
        error: false,
        stdout: include_str!("raw/dep_call_invalid_order.stdout"),
        stderr: include_str!("raw/dep_call_invalid_order.stderr"),
    });
}

#[test]
fn dep_call_firstclass() {
    check(Row {
        id: "dep_call_firstclass",
        source: r####"c(typeof(call),typeof(is.pairlist),deparse(.Primitive("call")("f",1L)))"####,
        error: false,
        stdout: include_str!("raw/dep_call_firstclass.stdout"),
        stderr: include_str!("raw/dep_call_firstclass.stderr"),
    });
}

#[test]
fn dep_pairlist_types() {
    check(Row {
        id: "dep_pairlist_types",
        source: r####"as.character(c(is.pairlist(NULL),is.pairlist(pairlist(x=1L)),is.pairlist(list(1L)),is.pairlist(expression(x)),is.pairlist(quote(f(x)))))"####,
        error: false,
        stdout: include_str!("raw/dep_pairlist_types.stdout"),
        stderr: include_str!("raw/dep_pairlist_types.stderr"),
    });
}

#[test]
fn dep_pairlist_arity() {
    check(Row {
        id: "dep_pairlist_arity",
        source: r####"is.pairlist(1L,2L)"####,
        error: true,
        stdout: include_str!("raw/dep_pairlist_arity.stdout"),
        stderr: include_str!("raw/dep_pairlist_arity.stderr"),
    });
}

#[test]
fn dep_pairlist_tag() {
    check(Row {
        id: "dep_pairlist_tag",
        source: r####"is.pairlist(wrong=NULL)"####,
        error: true,
        stdout: include_str!("raw/dep_pairlist_tag.stdout"),
        stderr: include_str!("raw/dep_pairlist_tag.stderr"),
    });
}

#[test]
fn dep_character_empty() {
    check(Row {
        id: "dep_character_empty",
        source: r####"as.character()"####,
        error: false,
        stdout: include_str!("raw/dep_character_empty.stdout"),
        stderr: include_str!("raw/dep_character_empty.stderr"),
    });
}

#[test]
fn dep_character_language() {
    check(Row {
        id: "dep_character_language",
        source: r####"as.character(list(quote(x+1L),quote(x),quote(f(a=1L))))"####,
        error: false,
        stdout: include_str!("raw/dep_character_language.stdout"),
        stderr: include_str!("raw/dep_character_language.stderr"),
    });
}

#[test]
fn dep_character_missing() {
    check(Row {
        id: "dep_character_missing",
        source: r####"as.character(list(NA_character_,"NA",NA_integer_,NULL))"####,
        error: false,
        stdout: include_str!("raw/dep_character_missing.stdout"),
        stderr: include_str!("raw/dep_character_missing.stderr"),
    });
}

#[test]
fn dep_character_values() {
    check(Row {
        id: "dep_character_values",
        source: r####"as.character(list(1L,TRUE,1.25,c(1L,3L),c("a","b")))"####,
        error: false,
        stdout: include_str!("raw/dep_character_values.stdout"),
        stderr: include_str!("raw/dep_character_values.stderr"),
    });
}

#[test]
fn dep_character_expression() {
    check(Row {
        id: "dep_character_expression",
        source: r####"as.character(expression(x+1L,x,1L))"####,
        error: false,
        stdout: include_str!("raw/dep_character_expression.stdout"),
        stderr: include_str!("raw/dep_character_expression.stderr"),
    });
}

#[test]
fn dep_character_classed() {
    check(Row {
        id: "dep_character_classed",
        source: r####"x<-list(quote(x+1L));class(x)<-"unknown";as.character(x)"####,
        error: false,
        stdout: include_str!("raw/dep_character_classed.stdout"),
        stderr: include_str!("raw/dep_character_classed.stderr"),
    });
}

#[test]
fn dep_character_tag() {
    check(Row {
        id: "dep_character_tag",
        source: r####"as.character(wrong=list(1L))"####,
        error: true,
        stdout: include_str!("raw/dep_character_tag.stdout"),
        stderr: include_str!("raw/dep_character_tag.stderr"),
    });
}
