use super::{check, Row};

#[test]
fn deparse_factor() {
    check(Row {
        id: "deparse_factor",
        source: r#"deparse(factor(c('b','a','b')))"#,
        error: false,
        stdout: include_str!("raw/deparse_factor.stdout"),
        stderr: include_str!("raw/deparse_factor.stderr"),
    });
}

#[test]
fn deparse_ordered_factor() {
    check(Row {
        id: "deparse_ordered_factor",
        source: r#"deparse(ordered(c('b','a','b')))"#,
        error: false,
        stdout: include_str!("raw/deparse_ordered_factor.stdout"),
        stderr: include_str!("raw/deparse_ordered_factor.stderr"),
    });
}

#[test]
fn deparse_dataframe() {
    check(Row {
        id: "deparse_dataframe",
        source: r#"deparse(data.frame(a=1:2,b=c('x','y')))"#,
        error: false,
        stdout: include_str!("raw/deparse_dataframe.stdout"),
        stderr: include_str!("raw/deparse_dataframe.stderr"),
    });
}

#[test]
fn deparse_character_attributes() {
    check(Row {
        id: "deparse_character_attributes",
        source: r#"x<-'text';attr(x,'note')<-'value';deparse(x)"#,
        error: false,
        stdout: include_str!("raw/deparse_character_attributes.stdout"),
        stderr: include_str!("raw/deparse_character_attributes.stderr"),
    });
}

#[test]
fn deparse_closure_attributes() {
    check(Row {
        id: "deparse_closure_attributes",
        source: r#"f<-function(x)x;attr(f,'note')<-2L;deparse(f)"#,
        error: false,
        stdout: include_str!("raw/deparse_closure_attributes.stdout"),
        stderr: include_str!("raw/deparse_closure_attributes.stderr"),
    });
}

#[test]
fn deparse_control_scompat() {
    check(Row {
        id: "deparse_control_scompat",
        source: r#"deparse(list(c(1L,3L),c(NA_real_,NA_real_),c(1,2.5)),control=c('keepInteger','keepNA','S_compatible'))"#,
        error: false,
        stdout: include_str!("raw/deparse_control_scompat.stdout"),
        stderr: include_str!("raw/deparse_control_scompat.stderr"),
    });
}

#[test]
fn deparse_warn_incomplete() {
    check(Row {
        id: "deparse_warn_incomplete",
        source: r#"deparse(globalenv(),control='warnIncomplete')"#,
        error: false,
        stdout: include_str!("raw/deparse_warn_incomplete.stdout"),
        stderr: include_str!("raw/deparse_warn_incomplete.stderr"),
    });
}

#[test]
fn deparse_names_empty() {
    check(Row {
        id: "deparse_names_empty",
        source: r#"x<-list();names(x)<-character();deparse(x)"#,
        error: false,
        stdout: include_str!("raw/deparse_names_empty.stdout"),
        stderr: include_str!("raw/deparse_names_empty.stderr"),
    });
}
