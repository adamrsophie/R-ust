use super::{check, Row};

#[test]
fn correction_function_one() {
    check(Row {
        id: "correction_function_one",
        source: r####"deparse(call("function",pairlist(x=1L)))"####,
        error: false,
        stdout: include_str!("raw/correction_function_one.stdout"),
        stderr: include_str!("raw/correction_function_one.stderr"),
    });
}

#[test]
fn correction_if_extra() {
    check(Row {
        id: "correction_if_extra",
        source: r####"deparse(call("if",TRUE,1L,2L,3L))"####,
        error: false,
        stdout: include_str!("raw/correction_if_extra.stdout"),
        stderr: include_str!("raw/correction_if_extra.stderr"),
    });
}

#[test]
fn correction_if_zero() {
    check(Row {
        id: "correction_if_zero",
        source: r####"deparse(call("if"))"####,
        error: false,
        stdout: include_str!("raw/correction_if_zero.stdout"),
        stderr: include_str!("raw/correction_if_zero.stderr"),
    });
}

#[test]
fn correction_if_one() {
    check(Row {
        id: "correction_if_one",
        source: r####"deparse(call("if",TRUE))"####,
        error: false,
        stdout: include_str!("raw/correction_if_one.stdout"),
        stderr: include_str!("raw/correction_if_one.stderr"),
    });
}

#[test]
fn correction_while_extra() {
    check(Row {
        id: "correction_while_extra",
        source: r####"deparse(call("while",FALSE,1L,2L))"####,
        error: false,
        stdout: include_str!("raw/correction_while_extra.stdout"),
        stderr: include_str!("raw/correction_while_extra.stderr"),
    });
}

#[test]
fn correction_while_zero() {
    check(Row {
        id: "correction_while_zero",
        source: r####"deparse(call("while"))"####,
        error: false,
        stdout: include_str!("raw/correction_while_zero.stdout"),
        stderr: include_str!("raw/correction_while_zero.stderr"),
    });
}

#[test]
fn correction_for_extra() {
    check(Row {
        id: "correction_for_extra",
        source: r####"deparse(call("for",quote(x),1L,2L,3L))"####,
        error: false,
        stdout: include_str!("raw/correction_for_extra.stdout"),
        stderr: include_str!("raw/correction_for_extra.stderr"),
    });
}

#[test]
fn correction_for_nonsymbol() {
    check(Row {
        id: "correction_for_nonsymbol",
        source: r####"deparse(call("for",1L,2L,3L))"####,
        error: false,
        stdout: include_str!("raw/correction_for_nonsymbol.stdout"),
        stderr: include_str!("raw/correction_for_nonsymbol.stderr"),
    });
}

#[test]
fn correction_for_zero() {
    check(Row {
        id: "correction_for_zero",
        source: r####"deparse(call("for"))"####,
        error: false,
        stdout: include_str!("raw/correction_for_zero.stdout"),
        stderr: include_str!("raw/correction_for_zero.stderr"),
    });
}

#[test]
fn correction_repeat_extra() {
    check(Row {
        id: "correction_repeat_extra",
        source: r####"deparse(call("repeat",1L,2L))"####,
        error: false,
        stdout: include_str!("raw/correction_repeat_extra.stdout"),
        stderr: include_str!("raw/correction_repeat_extra.stderr"),
    });
}

#[test]
fn correction_repeat_zero() {
    check(Row {
        id: "correction_repeat_zero",
        source: r####"deparse(call("repeat"))"####,
        error: false,
        stdout: include_str!("raw/correction_repeat_zero.stdout"),
        stderr: include_str!("raw/correction_repeat_zero.stderr"),
    });
}

#[test]
fn correction_paren_extra() {
    check(Row {
        id: "correction_paren_extra",
        source: r####"deparse(call("(",1L,2L))"####,
        error: false,
        stdout: include_str!("raw/correction_paren_extra.stdout"),
        stderr: include_str!("raw/correction_paren_extra.stderr"),
    });
}

#[test]
fn correction_paren_zero() {
    check(Row {
        id: "correction_paren_zero",
        source: r####"deparse(call("("))"####,
        error: false,
        stdout: include_str!("raw/correction_paren_zero.stdout"),
        stderr: include_str!("raw/correction_paren_zero.stderr"),
    });
}

#[test]
fn correction_break_extra() {
    check(Row {
        id: "correction_break_extra",
        source: r####"deparse(call("break",1L))"####,
        error: false,
        stdout: include_str!("raw/correction_break_extra.stdout"),
        stderr: include_str!("raw/correction_break_extra.stderr"),
    });
}

#[test]
fn correction_next_extra() {
    check(Row {
        id: "correction_next_extra",
        source: r####"deparse(call("next",1L))"####,
        error: false,
        stdout: include_str!("raw/correction_next_extra.stdout"),
        stderr: include_str!("raw/correction_next_extra.stderr"),
    });
}
