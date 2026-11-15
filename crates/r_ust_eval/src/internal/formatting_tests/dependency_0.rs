use super::{check, Row};

#[test]
fn dep_call_empty() {
    check(Row {
        id: "dep_call_empty",
        source: r####"call()"####,
        error: true,
        stdout: include_str!("raw/dep_call_empty.stdout"),
        stderr: include_str!("raw/dep_call_empty.stderr"),
    });
}

#[test]
fn dep_call_type() {
    check(Row {
        id: "dep_call_type",
        source: r####"call(1L,stop("UNREACHED"))"####,
        error: true,
        stdout: include_str!("raw/dep_call_type.stdout"),
        stderr: include_str!("raw/dep_call_type.stderr"),
    });
}

#[test]
fn dep_call_name_tag() {
    check(Row {
        id: "dep_call_name_tag",
        source: r####"call(wrong="f",stop("UNREACHED"))"####,
        error: true,
        stdout: include_str!("raw/dep_call_name_tag.stdout"),
        stderr: include_str!("raw/dep_call_name_tag.stderr"),
    });
}

#[test]
fn dep_call_name_empty() {
    check(Row {
        id: "dep_call_name_empty",
        source: r####"call("",stop("UNREACHED"))"####,
        error: true,
        stdout: include_str!("raw/dep_call_name_empty.stdout"),
        stderr: include_str!("raw/dep_call_name_empty.stderr"),
    });
}

#[test]
fn dep_call_name_na() {
    check(Row {
        id: "dep_call_name_na",
        source: r####"deparse(call(NA_character_,1L))"####,
        error: false,
        stdout: include_str!("raw/dep_call_name_na.stdout"),
        stderr: include_str!("raw/dep_call_name_na.stderr"),
    });
}

#[test]
fn dep_call_internal() {
    check(Row {
        id: "dep_call_internal",
        source: r####"call(".Internal",stop("UNREACHED"))"####,
        error: true,
        stdout: include_str!("raw/dep_call_internal.stdout"),
        stderr: include_str!("raw/dep_call_internal.stderr"),
    });
}

#[test]
fn dep_call_internal_call() {
    check(Row {
        id: "dep_call_internal_call",
        source: r####"tryCatch(call(".Internal"),error=function(e)deparse(conditionCall(e)))"####,
        error: false,
        stdout: include_str!("raw/dep_call_internal_call.stdout"),
        stderr: include_str!("raw/dep_call_internal_call.stderr"),
    });
}

#[test]
fn dep_call_empty_name_call() {
    check(Row {
        id: "dep_call_empty_name_call",
        source: r####"tryCatch(call(""),error=function(e)deparse(conditionCall(e)))"####,
        error: false,
        stdout: include_str!("raw/dep_call_empty_name_call.stdout"),
        stderr: include_str!("raw/dep_call_empty_name_call.stderr"),
    });
}

#[test]
fn dep_call_empty_argument() {
    check(Row {
        id: "dep_call_empty_argument",
        source: r####"call("f",,1L)"####,
        error: true,
        stdout: include_str!("raw/dep_call_empty_argument.stdout"),
        stderr: include_str!("raw/dep_call_empty_argument.stderr"),
    });
}

#[test]
fn dep_call_named_values() {
    check(Row {
        id: "dep_call_named_values",
        source: r####"deparse(call(name="f",a=1L,b=quote(x+1L)))"####,
        error: false,
        stdout: include_str!("raw/dep_call_named_values.stdout"),
        stderr: include_str!("raw/dep_call_named_values.stderr"),
    });
}

#[test]
fn dep_call_dots() {
    check(Row {
        id: "dep_call_dots",
        source: r####"f<-function(...)call("g",...);deparse(f(a=1L,b=quote(x)))"####,
        error: true,
        stdout: include_str!("raw/dep_call_dots.stdout"),
        stderr: include_str!("raw/dep_call_dots.stderr"),
    });
}

#[test]
fn dep_call_order() {
    check(Row {
        id: "dep_call_order",
        source: r####"n<-0L;x<-call({n<-n+1L;"f"},{n<-n+1L;n},{n<-n+1L;n});c(deparse(x),as.character(n))"####,
        error: false,
        stdout: include_str!("raw/dep_call_order.stdout"),
        stderr: include_str!("raw/dep_call_order.stderr"),
    });
}
