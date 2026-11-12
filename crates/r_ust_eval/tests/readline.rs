use r_ust_eval::{EvalError, Frontend, OutputEvent};
use r_ust_sexp::Sexp;

fn string(value: &str) -> Sexp {
    Sexp::String(vec![Some(value.to_string())])
}

fn output(frontend: &mut Frontend) -> String {
    frontend
        .runtime_mut()
        .take_output()
        .into_iter()
        .map(|event| match event {
            OutputEvent::Stdout(text) => text,
            OutputEvent::Stderr(text) => panic!("unexpected stderr: {text}"),
        })
        .collect()
}

fn forbidden_reader() -> std::io::Result<String> {
    panic!("batch readline must not read host input")
}

fn answer() -> std::io::Result<String> {
    Ok(" \tα two words\t \n".to_string())
}

#[test]
fn batch_calls_print_prompts_and_preserve_empty_prompt_cache_without_input() {
    let mut frontend = Frontend::default();
    frontend.runtime_mut().set_input_reader(forbidden_reader);
    assert_eq!(
        frontend
            .eval_source("readline(\"previous\");readline(as.character(NULL));readline(NULL)")
            .unwrap(),
        vec![string(""), string(""), string("")]
    );
    assert_eq!(output(&mut frontend), "previous\nprevious\n\n");
    for (code, prompt) in [
        ("readline()", "\n"),
        ("readline(pr=\"p: \")", "p: \n"),
        ("readline(prompt=)", "\n"),
        ("readline(c(\"first\",\"second\"))", "first\n"),
        ("readline(list(1,2))", "1\n"),
        ("readline(list(NULL))", "NULL\n"),
        ("readline(factor(\"b\",levels=c(\"a\",\"b\")))", "2\n"),
        ("readline(matrix(1:4,2))", "1\n"),
        ("readline(NA)", "NA\n"),
    ] {
        assert_eq!(
            frontend.eval_source(code).unwrap(),
            vec![string("")],
            "{code}"
        );
        assert_eq!(output(&mut frontend), prompt, "{code}");
    }
}

#[test]
fn interactive_input_is_host_owned_trimmed_and_visible() {
    let mut frontend = Frontend::default();
    frontend.runtime_mut().set_session_context(Vec::new(), true);
    assert!(matches!(
        frontend.eval_source("readline(\"ask: \")"),
        Err(EvalError::NotImplemented("host console input"))
    ));
    assert!(output(&mut frontend).is_empty());
    frontend.runtime_mut().set_input_reader(answer);
    assert_eq!(
        frontend
            .eval_source_visible("readline(invisible(\"ask: \"));readline(as.character(NULL))")
            .unwrap(),
        vec![string("α two words"), string("α two words")]
    );
    assert_eq!(output(&mut frontend), "ask: ");
    assert_eq!(
        Frontend::default().eval_source("readline()").unwrap(),
        vec![string("")]
    );
}

#[test]
fn arguments_match_before_forcing_and_control_flow_unwinds() {
    let mut frontend = Frontend::default();
    for code in [
        "readline(bogus=print(9))",
        "readline(\"p\",print(9))",
        "readline(pr=print(9),prompt=\"p\")",
        "readline(prompt=\"p\",prompt=print(9))",
    ] {
        assert!(
            matches!(frontend.eval_source(code), Err(EvalError::Arity(_))),
            "{code}"
        );
        assert!(output(&mut frontend).is_empty(), "{code}");
    }
    assert_eq!(
        frontend
            .eval_source("(function()readline(prompt=return(7)))()")
            .unwrap(),
        vec![Sexp::Real(vec![7.0])]
    );
    assert!(output(&mut frontend).is_empty());
    assert_eq!(
        frontend
            .eval_source("(function(...)readline(...))(pr=\"forward\")")
            .unwrap(),
        vec![string("")]
    );
    assert_eq!(output(&mut frontend), "forward\n");
    frontend
        .eval_source("invisible(lapply(c(\"a\",\"b\"),readline))")
        .unwrap();
    assert_eq!(output(&mut frontend), "a\nb\n");
}

#[test]
fn prompt_and_input_lengths_match_r_console_buffers() {
    fn long_answer() -> std::io::Result<String> {
        Ok(format!("  {}\n", "x".repeat(9000)))
    }
    fn eof() -> std::io::Result<String> {
        Ok(String::new())
    }
    let mut frontend = Frontend::default();
    frontend.eval_source("readline(strrep(\"p\",300))").unwrap();
    assert_eq!(output(&mut frontend), format!("{}\n", "p".repeat(255)));
    frontend.runtime_mut().set_session_context(Vec::new(), true);
    frontend.runtime_mut().set_input_reader(long_answer);
    assert_eq!(
        frontend.eval_source("readline()").unwrap(),
        vec![string(&"x".repeat(8190))]
    );
    frontend.runtime_mut().set_input_reader(eof);
    assert_eq!(
        frontend.eval_source("readline()").unwrap(),
        vec![string("")]
    );
}

#[test]
fn input_errors_propagate_and_output_failure_prevents_reading() {
    fn bad_input() -> std::io::Result<String> {
        Err(std::io::Error::other("reader failed"))
    }
    fn bad_output(_: OutputEvent) -> std::io::Result<()> {
        Err(std::io::Error::other("writer failed"))
    }
    let mut frontend = Frontend::default();
    frontend.runtime_mut().set_session_context(Vec::new(), true);
    frontend.runtime_mut().set_input_reader(bad_input);
    assert!(
        matches!(frontend.eval_source("readline(\"p\")"),Err(EvalError::Input(message)) if message == "reader failed")
    );
    assert_eq!(output(&mut frontend), "p");
    frontend.runtime_mut().set_input_reader(forbidden_reader);
    frontend.runtime_mut().set_output_sink(bad_output);
    assert!(
        matches!(frontend.eval_source("readline(\"p\")"),Err(EvalError::Output(message)) if message == "writer failed")
    );
}

#[test]
fn unsupported_prompt_deparsing_and_raw_byte_boundaries_are_explicit() {
    for code in ["readline(list(1:2))", "readline(strrep(\"é\",128))"] {
        assert!(
            matches!(
                Frontend::default().eval_source(code),
                Err(EvalError::NotImplemented(_))
            ),
            "{code}"
        );
    }
}
