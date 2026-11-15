use r_ust_eval::{Frontend, OutputEvent};
use r_ust_sexp::{AssignmentOperator, ListElement, Sexp};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn output(frontend: &mut Frontend) -> Result<String> {
    let mut text = String::new();
    for event in frontend.runtime_mut().take_output() {
        match event {
            OutputEvent::Stdout(value) => text.push_str(&value),
            event => return Err(format!("unexpected output: {event:?}").into()),
        }
    }
    Ok(text)
}

#[test]
fn nested_classed_auto_explicit_and_named_headers_match_pinned_r_bytes() -> Result {
    let mut frontend = Frontend::default();
    let source = include_str!("fixtures/list_names_print/nested-classed.R");
    // Source retention is already disabled in the frontend's reference profile.
    frontend.eval_source_printed(source.trim_start_matches("options(keep.source=FALSE)\n"))?;
    assert_eq!(
        output(&mut frontend)?.as_bytes(),
        include_bytes!("fixtures/list_names_print/nested-classed.stdout")
    );
    Ok(())
}

#[test]
fn explicit_named_variants_reset_only_classed_child_prefixes() -> Result {
    let reference = include_str!("fixtures/list_names_print/names-closure.stdout");
    for (length, classed, start, end) in [
        (1, true, "CLASSED_NA\n", "ORDER_NA\n"),
        (0, true, "CLASSED_EMPTY\n", "ORDER_EMPTY\n"),
        (1, false, "UNCLASSED_NA\n", "CLASS_ONLY\n"),
    ] {
        let mut frontend = Frontend::default();
        let value = Sexp::NamedList {
            elements: (0..length)
                .map(|_| ListElement {
                    name: None,
                    value: Sexp::Integer(vec![Some(1)]),
                })
                .collect(),
            class: classed.then(|| Box::new(Sexp::String(vec![Some("foo".into())]))),
            class_first: true,
        };
        frontend.runtime_mut().eval(&Sexp::Assignment {
            operator: AssignmentOperator::Left,
            target: "x".into(),
            value: Box::new(value),
        })?;
        frontend.eval_source_printed("list(x)")?;
        let expected = reference
            .split_once(start)
            .ok_or(start)?
            .1
            .split_once(end)
            .ok_or(end)?
            .0;
        assert_eq!(output(&mut frontend)?, expected);
    }
    Ok(())
}
