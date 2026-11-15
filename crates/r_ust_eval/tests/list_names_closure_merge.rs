use r_ust_eval::{Frontend, OutputEvent};
use r_ust_sexp::{AssignmentOperator, ListElement, Sexp};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn zero_source_closure_under_na_names_has_no_phantom_child_line() -> Result {
    let mut frontend = Frontend::default();
    let source = include_str!("fixtures/list_names_print/names-empty-closure.R");
    frontend.eval_source_printed(source.trim_start_matches("options(keep.source=FALSE)\n"))?;
    let mut text = String::new();
    for event in frontend.runtime_mut().take_output() {
        match event {
            OutputEvent::Stdout(value) => text.push_str(&value),
            event => return Err(format!("unexpected output: {event:?}").into()),
        }
    }
    assert_eq!(
        text.as_bytes(),
        include_bytes!("fixtures/list_names_print/names-empty-closure.stdout")
    );
    Ok(())
}

#[test]
fn classed_names_order_and_recursive_closure_identity_match_pinned_r_bytes() -> Result {
    let mut frontend = Frontend::default();
    let source = include_str!("fixtures/list_names_print/names-closure.R");
    frontend.eval_source_printed(source.trim_start_matches("options(keep.source=FALSE)\n"))?;
    let mut text = String::new();
    for event in frontend.runtime_mut().take_output() {
        match event {
            OutputEvent::Stdout(value) => text.push_str(&value),
            event => return Err(format!("unexpected output: {event:?}").into()),
        }
    }
    assert_eq!(
        text.as_bytes(),
        include_bytes!("fixtures/list_names_print/names-closure.stdout")
    );
    Ok(())
}

#[test]
fn historical_order_bits_only_matter_with_names_and_class() -> Result {
    for (names, classed, length) in [
        (true, true, 0),
        (true, true, 1),
        (true, false, 1),
        (false, true, 1),
    ] {
        let mut frontend = Frontend::default();
        for (name, first) in [("left", false), ("right", true)] {
            let elements = (0..length)
                .map(|_| ListElement {
                    name: None,
                    value: Sexp::Integer(vec![Some(1)]),
                })
                .collect();
            let value = if names {
                Sexp::NamedList {
                    elements,
                    class: classed.then(|| Box::new(Sexp::String(vec![Some("foo".into())]))),
                    class_first: first,
                }
            } else {
                Sexp::ClassedList {
                    elements,
                    class: Box::new(Sexp::String(vec![Some("foo".into())])),
                    class_first: first,
                }
            };
            frontend.runtime_mut().eval(&Sexp::Assignment {
                operator: AssignmentOperator::Left,
                target: name.into(),
                value: Box::new(value),
            })?;
        }
        let expected = Sexp::Logical(vec![
            Some(true),
            Some(!(names && classed)),
            Some(!(names && classed)),
        ]);
        assert_eq!(frontend.eval_source("c(identical(left,right),identical(left,right,attrib.as.set=FALSE),identical(right,left,attrib.as.set=FALSE))")?.last(), Some(&expected));
    }
    Ok(())
}
