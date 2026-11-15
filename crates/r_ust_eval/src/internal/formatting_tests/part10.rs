use super::{check, Row};

#[test]
fn deparse_expression() {
    check(Row {
        id: "deparse_expression",
        source: r#"deparse(expression(x+1L,1L,NA_character_))"#,
        error: false,
        stdout: include_str!("raw/deparse_expression.stdout"),
        stderr: include_str!("raw/deparse_expression.stderr"),
    });
}

#[test]
fn deparse_expression_names() {
    check(Row {
        id: "deparse_expression_names",
        source: r#"x<-expression(a=1L,2L);names(x)<-c(NA_character_,'');deparse(x)"#,
        error: false,
        stdout: include_str!("raw/deparse_expression_names.stdout"),
        stderr: include_str!("raw/deparse_expression_names.stderr"),
    });
}

#[test]
fn deparse_list() {
    check(Row {
        id: "deparse_list",
        source: r#"deparse(list(a=1L,b=c('x','y'),c=list(2L)))"#,
        error: false,
        stdout: include_str!("raw/deparse_list.stdout"),
        stderr: include_str!("raw/deparse_list.stderr"),
    });
}

#[test]
fn deparse_list_names() {
    check(Row {
        id: "deparse_list_names",
        source: r#"x<-list(1L,2L,3L);names(x)<-c('',NA_character_,'a b');deparse(x)"#,
        error: false,
        stdout: include_str!("raw/deparse_list_names.stdout"),
        stderr: include_str!("raw/deparse_list_names.stderr"),
    });
}

#[test]
fn deparse_pairlist() {
    check(Row {
        id: "deparse_pairlist",
        source: r#"deparse(pairlist(x=1L,y=quote(a+b)))"#,
        error: false,
        stdout: include_str!("raw/deparse_pairlist.stdout"),
        stderr: include_str!("raw/deparse_pairlist.stderr"),
    });
}

#[test]
fn deparse_primitive() {
    check(Row {
        id: "deparse_primitive",
        source: r#"deparse(sum)"#,
        error: false,
        stdout: include_str!("raw/deparse_primitive.stdout"),
        stderr: include_str!("raw/deparse_primitive.stderr"),
    });
}

#[test]
fn deparse_environment() {
    check(Row {
        id: "deparse_environment",
        source: r#"deparse(globalenv())"#,
        error: false,
        stdout: include_str!("raw/deparse_environment.stdout"),
        stderr: include_str!("raw/deparse_environment.stderr"),
    });
}

#[test]
fn deparse_control_empty() {
    check(Row {
        id: "deparse_control_empty",
        source: r#"deparse(c(1L,NA_integer_),control=character())"#,
        error: false,
        stdout: include_str!("raw/deparse_control_empty.stdout"),
        stderr: include_str!("raw/deparse_control_empty.stderr"),
    });
}
