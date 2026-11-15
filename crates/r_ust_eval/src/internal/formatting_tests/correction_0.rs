use super::{check, Row};

#[test]
fn correction_primitive_backtick_false() {
    check(Row {
        id: "correction_primitive_backtick_false",
        source: r####"deparse(call("+",1L,2L,3L),backtick=FALSE)"####,
        error: false,
        stdout: include_str!("raw/correction_primitive_backtick_false.stdout"),
        stderr: include_str!("raw/correction_primitive_backtick_false.stderr"),
    });
}

#[test]
fn correction_primitive_backtick_true() {
    check(Row {
        id: "correction_primitive_backtick_true",
        source: r####"deparse(call("+",1L,2L,3L),backtick=TRUE)"####,
        error: false,
        stdout: include_str!("raw/correction_primitive_backtick_true.stdout"),
        stderr: include_str!("raw/correction_primitive_backtick_true.stderr"),
    });
}

#[test]
fn correction_primitive_scompat() {
    check(Row {
        id: "correction_primitive_scompat",
        source: r####"deparse(call("+",1L,2L,3L),backtick=FALSE,control="S_compatible")"####,
        error: false,
        stdout: include_str!("raw/correction_primitive_scompat.stdout"),
        stderr: include_str!("raw/correction_primitive_scompat.stderr"),
    });
}

#[test]
fn correction_user_binop() {
    check(Row {
        id: "correction_user_binop",
        source: r####"deparse(call("%foo%",a=1L,b=2L),backtick=FALSE)"####,
        error: false,
        stdout: include_str!("raw/correction_user_binop.stdout"),
        stderr: include_str!("raw/correction_user_binop.stderr"),
    });
}

#[test]
fn correction_ordinary_caller() {
    check(Row {
        id: "correction_ordinary_caller",
        source: r####"deparse(call("a b",1L),backtick=FALSE)"####,
        error: false,
        stdout: include_str!("raw/correction_ordinary_caller.stdout"),
        stderr: include_str!("raw/correction_ordinary_caller.stderr"),
    });
}

#[test]
fn correction_ordinary_scompat() {
    check(Row {
        id: "correction_ordinary_scompat",
        source: r####"deparse(call("a b",1L),backtick=FALSE,control="S_compatible")"####,
        error: false,
        stdout: include_str!("raw/correction_ordinary_scompat.stdout"),
        stderr: include_str!("raw/correction_ordinary_scompat.stderr"),
    });
}

#[test]
fn correction_subassign() {
    check(Row {
        id: "correction_subassign",
        source: r####"deparse(call("[<-",quote(x),1L,2L),backtick=FALSE)"####,
        error: false,
        stdout: include_str!("raw/correction_subassign.stdout"),
        stderr: include_str!("raw/correction_subassign.stderr"),
    });
}

#[test]
fn correction_dollar_utf8() {
    check(Row {
        id: "correction_dollar_utf8",
        source: r####"deparse(call("$",quote(x),"é"))"####,
        error: false,
        stdout: include_str!("raw/correction_dollar_utf8.stdout"),
        stderr: include_str!("raw/correction_dollar_utf8.stderr"),
    });
}

#[test]
fn correction_dollar_ascii() {
    check(Row {
        id: "correction_dollar_ascii",
        source: r####"deparse(call("$",quote(x),"a"))"####,
        error: false,
        stdout: include_str!("raw/correction_dollar_ascii.stdout"),
        stderr: include_str!("raw/correction_dollar_ascii.stderr"),
    });
}

#[test]
fn correction_dollar_space() {
    check(Row {
        id: "correction_dollar_space",
        source: r####"deparse(call("$",quote(x),"a b"))"####,
        error: false,
        stdout: include_str!("raw/correction_dollar_space.stdout"),
        stderr: include_str!("raw/correction_dollar_space.stderr"),
    });
}

#[test]
fn correction_dollar_quote() {
    check(Row {
        id: "correction_dollar_quote",
        source: r####"deparse(call("$",quote(x),"a\"b"))"####,
        error: false,
        stdout: include_str!("raw/correction_dollar_quote.stdout"),
        stderr: include_str!("raw/correction_dollar_quote.stderr"),
    });
}

#[test]
fn correction_dollar_backtick_false() {
    check(Row {
        id: "correction_dollar_backtick_false",
        source: r####"deparse(call("$",quote(x),"é"),backtick=FALSE)"####,
        error: false,
        stdout: include_str!("raw/correction_dollar_backtick_false.stdout"),
        stderr: include_str!("raw/correction_dollar_backtick_false.stderr"),
    });
}

#[test]
fn correction_dollar_warning() {
    check(Row {
        id: "correction_dollar_warning",
        source: r####"warning(simpleWarning("message",call("$",quote(x),"é")));"ok""####,
        error: false,
        stdout: include_str!("raw/correction_dollar_warning.stdout"),
        stderr: include_str!("raw/correction_dollar_warning.stderr"),
    });
}

#[test]
fn correction_function_two() {
    check(Row {
        id: "correction_function_two",
        source: r####"deparse(call("function",pairlist(x=1L),quote(x)))"####,
        error: false,
        stdout: include_str!("raw/correction_function_two.stdout"),
        stderr: include_str!("raw/correction_function_two.stderr"),
    });
}

#[test]
fn correction_function_integer_source() {
    check(Row {
        id: "correction_function_integer_source",
        source: r####"deparse(call("function",pairlist(x=1L),quote(x),7L),control=character())"####,
        error: false,
        stdout: include_str!("raw/correction_function_integer_source.stdout"),
        stderr: include_str!("raw/correction_function_integer_source.stderr"),
    });
}

#[test]
fn correction_function_default_integer() {
    check(Row {
        id: "correction_function_default_integer",
        source: r####"deparse(call("function",pairlist(x=1L),quote(x),7L))"####,
        error: false,
        stdout: include_str!("raw/correction_function_default_integer.stdout"),
        stderr: include_str!("raw/correction_function_default_integer.stderr"),
    });
}

#[test]
fn correction_function_extra() {
    check(Row {
        id: "correction_function_extra",
        source: r####"deparse(call("function",pairlist(x=1L),quote(x),NULL,9L))"####,
        error: false,
        stdout: include_str!("raw/correction_function_extra.stdout"),
        stderr: include_str!("raw/correction_function_extra.stderr"),
    });
}

#[test]
fn correction_function_ignore_source() {
    check(Row {
        id: "correction_function_ignore_source",
        source: r####"deparse(call("function",pairlist(x=1L),quote(x),"source text"),control=character())"####,
        error: false,
        stdout: include_str!("raw/correction_function_ignore_source.stdout"),
        stderr: include_str!("raw/correction_function_ignore_source.stderr"),
    });
}

#[test]
fn correction_function_source() {
    check(Row {
        id: "correction_function_source",
        source: r####"deparse(call("function",pairlist(x=1L),quote(x),c("function(x)","x")))"####,
        error: false,
        stdout: include_str!("raw/correction_function_source.stdout"),
        stderr: include_str!("raw/correction_function_source.stderr"),
    });
}

#[test]
fn correction_function_zero() {
    check(Row {
        id: "correction_function_zero",
        source: r####"deparse(call("function"))"####,
        error: false,
        stdout: include_str!("raw/correction_function_zero.stdout"),
        stderr: include_str!("raw/correction_function_zero.stderr"),
    });
}
