use super::{check, Row};

#[test]
fn dep_wrapper_call_matrix() {
    check(Row {
        id: "dep_wrapper_call_matrix",
        source: r####"deparse(call(matrix("f",1L,1L),1L))"####,
        error: false,
        stdout: include_str!("raw/dep_wrapper_call_matrix.stdout"),
        stderr: include_str!("raw/dep_wrapper_call_matrix.stderr"),
    });
}

#[test]
fn dep_wrapper_call_matrix_na() {
    check(Row {
        id: "dep_wrapper_call_matrix_na",
        source: r####"deparse(call(matrix(NA_character_,1L,1L),1L))"####,
        error: false,
        stdout: include_str!("raw/dep_wrapper_call_matrix_na.stdout"),
        stderr: include_str!("raw/dep_wrapper_call_matrix_na.stderr"),
    });
}

#[test]
fn dep_wrapper_call_matrix_long() {
    check(Row {
        id: "dep_wrapper_call_matrix_long",
        source: r####"call(matrix(c("f","g"),1L,2L),stop("UNREACHED"))"####,
        error: true,
        stdout: include_str!("raw/dep_wrapper_call_matrix_long.stdout"),
        stderr: include_str!("raw/dep_wrapper_call_matrix_long.stderr"),
    });
}

#[test]
fn dep_wrapper_character_element() {
    check(Row {
        id: "dep_wrapper_character_element",
        source: r####"as.character(list(matrix("x",1L,1L)))"####,
        error: false,
        stdout: include_str!("raw/dep_wrapper_character_element.stdout"),
        stderr: include_str!("raw/dep_wrapper_character_element.stderr"),
    });
}

#[test]
fn dep_wrapper_character_element_na() {
    check(Row {
        id: "dep_wrapper_character_element_na",
        source: r####"as.character(list(matrix(NA_character_,1L,1L)))"####,
        error: false,
        stdout: include_str!("raw/dep_wrapper_character_element_na.stdout"),
        stderr: include_str!("raw/dep_wrapper_character_element_na.stderr"),
    });
}

#[test]
fn dep_wrapper_character_matrix_list() {
    check(Row {
        id: "dep_wrapper_character_matrix_list",
        source: r####"as.character(matrix(list(NA_character_),1L,1L))"####,
        error: false,
        stdout: include_str!("raw/dep_wrapper_character_matrix_list.stdout"),
        stderr: include_str!("raw/dep_wrapper_character_matrix_list.stderr"),
    });
}

#[test]
fn dep_wrapper_character_dataframe() {
    check(Row {
        id: "dep_wrapper_character_dataframe",
        source: r####"as.character(data.frame(x=NA_character_))"####,
        error: false,
        stdout: include_str!("raw/dep_wrapper_character_dataframe.stdout"),
        stderr: include_str!("raw/dep_wrapper_character_dataframe.stderr"),
    });
}

#[test]
fn dep_wrapper_character_dataframe_columns() {
    check(Row {
        id: "dep_wrapper_character_dataframe_columns",
        source: r####"as.character(data.frame(x=c(1L,3L),y=c("a","b")))"####,
        error: false,
        stdout: include_str!("raw/dep_wrapper_character_dataframe_columns.stdout"),
        stderr: include_str!("raw/dep_wrapper_character_dataframe_columns.stderr"),
    });
}
