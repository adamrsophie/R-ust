use std::sync::atomic::{AtomicUsize, Ordering};

use r_ust_eval::{EvalError, Frontend, OutputEvent};
use r_ust_sexp::Sexp;

const GENERIC: &str = "NAs introduced by coercion";
const RANGE: &str = "NAs introduced by coercion to integer range";
static WRITES: AtomicUsize = AtomicUsize::new(0);

fn failing_sink(_: OutputEvent) -> std::io::Result<()> {
    WRITES.fetch_add(1, Ordering::SeqCst);
    Err(std::io::Error::other("closed coercion warning output"))
}

#[test]
fn atomic_and_list_warning_order_matches_r() {
    for (source, warnings) in [
        (
            "as.integer(c('bad','2147483648','bad'))",
            vec![GENERIC, RANGE],
        ),
        (
            "as.integer(c('2147483648','bad','bad'))",
            vec![GENERIC, RANGE],
        ),
        (
            "as.integer(list('bad','2147483648','bad'))",
            vec![GENERIC, RANGE, GENERIC],
        ),
        (
            "as.integer(list('2147483648','bad','bad'))",
            vec![RANGE, GENERIC, GENERIC],
        ),
    ] {
        for warn in [-1, 0, 1, 2, 3] {
            let mut frontend = Frontend::default();
            frontend
                .runtime_mut()
                .set_option("warn", Sexp::Integer(vec![Some(warn)]))
                .unwrap();
            let result = frontend.eval_source_printed(source);
            let value = OutputEvent::Stdout("[1] NA NA NA\n".into());
            let expected = match warn {
                -1 => vec![value],
                0 => {
                    let lines = warnings
                        .iter()
                        .enumerate()
                        .map(|(i, warning)| format!("{}: {warning} \n", i + 1))
                        .collect::<String>();
                    vec![
                        value,
                        OutputEvent::Stderr(format!("Warning messages:\n{lines}")),
                    ]
                }
                1 => warnings
                    .iter()
                    .map(|warning| OutputEvent::Stderr(format!("Warning: {warning}\n")))
                    .chain([value])
                    .collect(),
                _ => Vec::new(),
            };
            assert_eq!(
                frontend.runtime_mut().take_output(),
                expected,
                "{source}; warn={warn}"
            );
            if warn >= 2 {
                assert_eq!(
                    result.unwrap_err(),
                    EvalError::Domain(format!("(converted from warning) {}", warnings[0]))
                );
            } else {
                result.unwrap();
            }
        }
    }
}

#[test]
fn promotion_and_sink_failure_stop_delivery() {
    let mut frontend = Frontend::default();
    frontend.eval_source("options(warn=2)").unwrap();
    assert_eq!(
        frontend
            .eval_source("as.integer(list('2147483648','bad'));after<-1")
            .unwrap_err(),
        EvalError::Domain(format!("(converted from warning) {RANGE}"))
    );
    assert_eq!(frontend.runtime().get("after").unwrap(), None);
    assert!(frontend.runtime_mut().take_output().is_empty());

    for warn in [0, 1] {
        WRITES.store(0, Ordering::SeqCst);
        let mut frontend = Frontend::default();
        frontend
            .runtime_mut()
            .set_option("warn", Sexp::Integer(vec![Some(warn)]))
            .unwrap();
        frontend.runtime_mut().set_output_sink(failing_sink);
        assert!(matches!(
            frontend.eval_source("as.integer(list('2147483648','bad','bad'))"),
            Err(EvalError::Output(_))
        ));
        assert_eq!(WRITES.load(Ordering::SeqCst), 1, "warn={warn}");
        frontend
            .runtime_mut()
            .finish_parse_warnings(Ok(()))
            .unwrap();
        assert_eq!(
            WRITES.load(Ordering::SeqCst),
            1,
            "finish must not retry warn={warn}"
        );
    }
}
