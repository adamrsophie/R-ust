use super::{check, Row};

#[test]
fn sprintf_internal_rebound_character() {
    check(Row {
        id: "sprintf_internal_rebound_character",
        source: r####"as.character<-function(x)'changed';.Internal(sprintf('%s',1L))"####,
        error: false,
        stdout: include_str!("raw/sprintf_internal_rebound_character.stdout"),
        stderr: include_str!("raw/sprintf_internal_rebound_character.stderr"),
    });
}

#[test]
fn sprintf_internal_rebound_double() {
    check(Row {
        id: "sprintf_internal_rebound_double",
        source: r####"as.double<-function(x)2.5;.Internal(sprintf('%.1f',1L))"####,
        error: false,
        stdout: include_str!("raw/sprintf_internal_rebound_double.stdout"),
        stderr: include_str!("raw/sprintf_internal_rebound_double.stderr"),
    });
}

#[test]
fn sprintf_internal_rebound_empty() {
    check(Row {
        id: "sprintf_internal_rebound_empty",
        source: r####"as.character<-function(x)character();.Internal(sprintf('%s',1L))"####,
        error: true,
        stdout: include_str!("raw/sprintf_internal_rebound_empty.stdout"),
        stderr: include_str!("raw/sprintf_internal_rebound_empty.stderr"),
    });
}

#[test]
fn sprintf_internal_coercion_once() {
    check(Row {
        id: "sprintf_internal_coercion_once",
        source: r####"n<-0L;as.character<-function(x){n<<-n+1L;'changed'};c(.Internal(sprintf('%1$s %1$s',1L)),.Internal(sprintf('%d',n)))"####,
        error: false,
        stdout: include_str!("raw/sprintf_internal_coercion_once.stdout"),
        stderr: include_str!("raw/sprintf_internal_coercion_once.stderr"),
    });
}

#[test]
fn sprintf_internal_coercion_error() {
    check(Row {
        id: "sprintf_internal_coercion_error",
        source: r####"as.character<-function(x)stop('coercion failed');.Internal(sprintf('%s',1L))"####,
        error: true,
        stdout: include_str!("raw/sprintf_internal_coercion_error.stdout"),
        stderr: include_str!("raw/sprintf_internal_coercion_error.stderr"),
    });
}
