use super::{check, Row};

#[test]
fn deparse_symbol() {
    check(Row {
        id: "deparse_symbol",
        source: r#"deparse(quote(`a b`))"#,
        error: false,
        stdout: include_str!("raw/deparse_symbol.stdout"),
        stderr: include_str!("raw/deparse_symbol.stderr"),
    });
}

#[test]
fn deparse_symbol_backtick() {
    check(Row {
        id: "deparse_symbol_backtick",
        source: r#"deparse(quote(`a b`),backtick=TRUE)"#,
        error: false,
        stdout: include_str!("raw/deparse_symbol_backtick.stdout"),
        stderr: include_str!("raw/deparse_symbol_backtick.stderr"),
    });
}

#[test]
fn deparse_call() {
    check(Row {
        id: "deparse_call",
        source: r#"deparse(quote(f(a=1L,b=x+2)))"#,
        error: false,
        stdout: include_str!("raw/deparse_call.stdout"),
        stderr: include_str!("raw/deparse_call.stderr"),
    });
}

#[test]
fn deparse_call_missing() {
    check(Row {
        id: "deparse_call_missing",
        source: r#"deparse(quote(f(,x,)))"#,
        error: false,
        stdout: include_str!("raw/deparse_call_missing.stdout"),
        stderr: include_str!("raw/deparse_call_missing.stderr"),
    });
}

#[test]
fn deparse_call_nonstandard() {
    check(Row {
        id: "deparse_call_nonstandard",
        source: r#"deparse(quote(`a b`(`x y`=1L)))"#,
        error: false,
        stdout: include_str!("raw/deparse_call_nonstandard.stdout"),
        stderr: include_str!("raw/deparse_call_nonstandard.stderr"),
    });
}

#[test]
fn deparse_call_no_backtick() {
    check(Row {
        id: "deparse_call_no_backtick",
        source: r#"deparse(quote(`a b`(`x y`=1L)),backtick=FALSE)"#,
        error: false,
        stdout: include_str!("raw/deparse_call_no_backtick.stdout"),
        stderr: include_str!("raw/deparse_call_no_backtick.stderr"),
    });
}

#[test]
fn deparse_block() {
    check(Row {
        id: "deparse_block",
        source: r#"deparse(quote({x<-1L;if(x>0L)x+1L else 0L}))"#,
        error: false,
        stdout: include_str!("raw/deparse_block.stdout"),
        stderr: include_str!("raw/deparse_block.stderr"),
    });
}

#[test]
fn deparse_function() {
    check(Row {
        id: "deparse_function",
        source: r#"deparse(function(x=1L,...) {x+1L})"#,
        error: false,
        stdout: include_str!("raw/deparse_function.stdout"),
        stderr: include_str!("raw/deparse_function.stderr"),
    });
}
