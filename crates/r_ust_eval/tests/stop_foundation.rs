use r_ust_eval::{EvalError, Frontend, OutputEvent};
use r_ust_sexp::Sexp;

fn decode(text: &str) -> String {
    if text == "-" {
        return String::new();
    }
    assert!(!text.is_empty() && text.len().is_multiple_of(2));
    assert!(text.bytes().all(|byte| byte.is_ascii_hexdigit()));
    String::from_utf8(
        text.as_bytes()
            .as_chunks::<2>()
            .0
            .iter()
            .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
            .collect(),
    )
    .unwrap()
}

#[test]
fn wrapper_and_aliases_match_independent_r_fixture() {
    let mut failures = Vec::new();
    let mut checked = 0;
    let mut limited = 0;
    let mut lines = include_str!("fixtures/stop-r-4.6.0.tsv").lines();
    assert_eq!(lines.next(), Some("# R version 4.6.0 (2026-04-24) aarch64-apple-darwin23 LDOUBLE 8 JIT 0 DIRECT_TOP_LEVEL"));
    for (index, line) in lines.enumerate() {
        let fields: Vec<_> = line.split('\t').collect();
        assert_eq!(fields.len(), 10);
        assert_eq!(fields[0].parse::<usize>().unwrap(), index + 1);
        match fields[2] {
            "limited-condition" => {
                assert_eq!(fields[1], "condition-object");
                limited += 1;
                continue;
            }
            "stop" | "error" | "value" => {}
            other => panic!("unknown fixture lane {other}"),
        }
        checked += 1;
        let mut frontend = Frontend::default();
        let result = frontend.eval_source(fields[3]);
        let expected_message = decode(fields[6]);
        let matched = match (&result, fields[2]) {
            (
                Err(EvalError::Stop {
                    message,
                    call_requested,
                }),
                "stop",
            ) => message == &expected_message && *call_requested == (fields[7] != "-"),
            (Err(error), "error") => {
                let message = match error {
                    EvalError::Domain(message)
                    | EvalError::Arity(message)
                    | EvalError::Type(message) => message.clone(),
                    error => error.to_string(),
                };
                message == expected_message
            }
            (Ok(values), "value") => match (values.last(), fields[4]) {
                (Some(Sexp::Integer(values)), "integer") => {
                    values
                        .iter()
                        .map(|value| value.unwrap().to_string())
                        .collect::<Vec<_>>()
                        .join("|")
                        == decode(fields[5])
                }
                (Some(Sexp::String(values)), "character") => {
                    values
                        .iter()
                        .map(|value| value.clone().unwrap_or_else(|| "NA".into()))
                        .collect::<Vec<_>>()
                        .join("|")
                        == decode(fields[5])
                }
                _ => false,
            },
            _ => false,
        };
        let mut stdout = String::new();
        let mut stderr = String::new();
        for event in frontend.runtime_mut().take_output() {
            match event {
                OutputEvent::Stdout(text) => stdout.push_str(&text),
                OutputEvent::StdoutBytes(bytes) => {
                    stdout.push_str(std::str::from_utf8(&bytes).unwrap())
                }
                OutputEvent::Stderr(text) => stderr.push_str(&text),
            }
        }
        if !matched || stdout != decode(fields[9]) || stderr != decode(fields[8]) {
            failures.push(format!("{} {}: {result:?}, stdout={stdout:?}, stderr={stderr:?}, expected_message={expected_message:?}", fields[0], fields[1]));
        }
    }
    assert_eq!(checked, 120);
    assert_eq!(limited, 1);
    assert_eq!(checked + limited, 121);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn transport_keeps_stop_payload_and_restores_frontend() {
    let mut frontend = Frontend::default();
    frontend
        .runtime_mut()
        .record_parse_warnings(&["pending".into()])
        .unwrap();
    let error = frontend
        .eval_source("cat('before');stop('boom',call.=FALSE);cat('after')")
        .unwrap_err();
    assert_eq!(
        error,
        EvalError::WithWarnings {
            error: Box::new(EvalError::Stop {
                message: "boom".into(),
                call_requested: false
            }),
            warnings: "Warning message:\npending \n".into(),
        }
    );
    assert_eq!(
        frontend.runtime_mut().take_output(),
        [OutputEvent::Stdout("before".into())]
    );
    assert_eq!(
        frontend.eval_source("x<-42L;x").unwrap().last(),
        Some(&Sexp::Integer(vec![Some(42)]))
    );
}

#[test]
fn transport_host_output_failure_precedes_stop() {
    fn fail(_: OutputEvent) -> std::io::Result<()> {
        Err(std::io::Error::other("sink failed"))
    }
    let mut frontend = Frontend::default();
    frontend.runtime_mut().set_output_sink(fail);
    assert_eq!(
        frontend.eval_source("stop({cat('x');'boom'})"),
        Err(EvalError::Output("sink failed".into()))
    );
    assert_eq!(
        frontend.eval_source("42L").unwrap(),
        [Sexp::Integer(vec![Some(42)])]
    );
}

#[test]
fn base_internal_lookup_hides_base_frames_from_caller_promises() {
    for (source, symbol) in [
        ("stop(call.)", "call."),
        ("stop(domain)", "domain"),
        ("inherits(what,'logical')", "what"),
        (".makeMessage(appendLF)", "appendLF"),
    ] {
        let mut frontend = Frontend::default();
        assert_eq!(
            frontend.eval_source(source),
            Err(EvalError::UndefinedSymbol(symbol.into())),
            "{source}"
        );
    }
}

#[test]
fn base_internal_lookup_superassignment_cannot_rebind_base_formals() {
    let mut frontend = Frontend::default();
    assert_eq!(
        frontend.eval_source("stop({call.<<-FALSE;'x'})"),
        Err(EvalError::Stop {
            message: "x".into(),
            call_requested: true
        })
    );
    assert_eq!(
        frontend.eval_source("call.").unwrap(),
        [Sexp::Logical(vec![Some(false)])]
    );
}

#[test]
fn base_internal_unwind_restores_lookup_after_errors_and_return() {
    for (source, expected) in [
        (
            "stop('x')",
            Err(EvalError::Stop {
                message: "x".into(),
                call_requested: true,
            }),
        ),
        (
            "stop('x',call.=TRUE,call.=FALSE)",
            Err(EvalError::Arity(
                "formal argument \"call.\" matched by multiple actual arguments".into(),
            )),
        ),
        (
            "stop('x',domain=TRUE)",
            Err(EvalError::Domain("invalid 'domain' value".into())),
        ),
        (
            "(function(){stop(return(42L));99L})()",
            Ok(vec![Sexp::Integer(vec![Some(42)])]),
        ),
    ] {
        let mut frontend = Frontend::default();
        assert_eq!(frontend.eval_source(source), expected, "{source}");
        assert_eq!(
            frontend
                .eval_source("inherits<-function(...)42L;inherits(1L,'x')")
                .unwrap()
                .last(),
            Some(&Sexp::Integer(vec![Some(42)])),
            "{source}"
        );
    }
}
