use super::{check, Row};

#[test]
fn deparse_string_call_head() {
    check(Row {
        id: "deparse_string_call_head",
        source: r####"deparse(as.call(list('sum',1L)))"####,
        error: false,
        stdout: include_str!("raw/deparse_string_call_head.stdout"),
        stderr: include_str!("raw/deparse_string_call_head.stderr"),
    });
}

#[test]
fn deparse_vector_call_head() {
    check(Row {
        id: "deparse_vector_call_head",
        source: r####"deparse(as.call(list(c('a','b'),1L)))"####,
        error: false,
        stdout: include_str!("raw/deparse_vector_call_head.stdout"),
        stderr: include_str!("raw/deparse_vector_call_head.stderr"),
    });
}

#[test]
fn deparse_integer_call_head() {
    check(Row {
        id: "deparse_integer_call_head",
        source: r####"deparse(as.call(list(1L,2L)))"####,
        error: false,
        stdout: include_str!("raw/deparse_integer_call_head.stdout"),
        stderr: include_str!("raw/deparse_integer_call_head.stderr"),
    });
}

#[test]
fn deparse_primitive_call_head() {
    check(Row {
        id: "deparse_primitive_call_head",
        source: r####"deparse(as.call(list(sum,1L)))"####,
        error: false,
        stdout: include_str!("raw/deparse_primitive_call_head.stdout"),
        stderr: include_str!("raw/deparse_primitive_call_head.stderr"),
    });
}

#[test]
fn deparse_closure_call_head() {
    check(Row {
        id: "deparse_closure_call_head",
        source: r####"deparse(as.call(list(function(x)x,1L)))"####,
        error: false,
        stdout: include_str!("raw/deparse_closure_call_head.stdout"),
        stderr: include_str!("raw/deparse_closure_call_head.stderr"),
    });
}

#[test]
fn deparse_names_missing_literal() {
    check(Row {
        id: "deparse_names_missing_literal",
        source: r####"x<-list(1L,2L,3L);names(x)<-c(NA_character_,'NA','');deparse(x)"####,
        error: false,
        stdout: include_str!("raw/deparse_names_missing_literal.stdout"),
        stderr: include_str!("raw/deparse_names_missing_literal.stderr"),
    });
}

#[test]
fn deparse_expression_empty_names() {
    check(Row {
        id: "deparse_expression_empty_names",
        source: r####"x<-expression();names(x)<-character();deparse(x)"####,
        error: false,
        stdout: include_str!("raw/deparse_expression_empty_names.stdout"),
        stderr: include_str!("raw/deparse_expression_empty_names.stderr"),
    });
}
