use super::fixtures::Row;
use r_ust_eval::{Frontend, OutputEvent};
use r_ust_sexp::{ListElement, Sexp};

pub(super) fn last(frontend: &mut Frontend, source: &str) -> Sexp {
    frontend.eval_source(source).unwrap().pop().unwrap()
}

pub(super) fn strings(values: &[&str]) -> Sexp {
    Sexp::String(values.iter().map(|value| Some((*value).into())).collect())
}

pub(super) fn plain(value: &Sexp) -> Sexp {
    let value = match value {
        Sexp::ListObject(object) => object.value().unwrap(),
        Sexp::PairListObject(object) => Sexp::pairlist(object.elements().unwrap()),
        other => other.clone(),
    };
    if let Some(view) = value.ordinary_list() {
        return view.rebuild(elements(view.elements), view.names_present);
    }
    match value {
        Sexp::PairList(values) => Sexp::pairlist(elements(values.as_slice())),
        Sexp::Expression {
            elements: values,
            names_present,
        } => Sexp::Expression {
            elements: elements(&values),
            names_present,
        },
        other => other,
    }
}

fn elements(values: &[ListElement]) -> Vec<ListElement> {
    values
        .iter()
        .map(|element| ListElement {
            name: element.name.clone(),
            value: plain(&element.value),
        })
        .collect()
}

pub(super) fn effects(frontend: &mut Frontend, row: &Row) {
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    for event in frontend.runtime_mut().take_output() {
        match event {
            OutputEvent::Stdout(text) => stdout.extend_from_slice(text.as_bytes()),
            OutputEvent::StdoutBytes(bytes) => stdout.extend(bytes),
            OutputEvent::Stderr(text) => stderr.extend_from_slice(text.as_bytes()),
        }
    }
    assert_eq!(stdout, row.stdout.as_bytes(), "{} effects", row.id);
    let expected = match row.id {
        "warning" => "Warning in evalq({ : subject warning\n",
        "partial_warning" => {
            "Warning in eval(ex = quote(2L)) :\n  partial argument match of 'ex' to 'expr'\n"
        }
        _ => "",
    };
    assert_eq!(stderr, expected.as_bytes(), "{} warning sink", row.id);
}

pub(super) fn metadata(frontend: &mut Frontend, value: &Sexp, row: &Row) {
    assert_eq!(
        r_ust_eval::format_sexp(value).unwrap(),
        row.print,
        "{} print",
        row.id
    );
    let Sexp::Environment(frame) = last(frontend, "globalenv()") else {
        panic!("missing global environment")
    };
    frame.insert(".d3_value".into(), value).unwrap();
    assert_eq!(
        last(frontend, "typeof(.d3_value)"),
        strings(&[row.expected_type.unwrap()]),
        "{} typeof",
        row.id
    );
    assert_eq!(
        last(frontend, "class(.d3_value)"),
        strings(row.class),
        "{} class",
        row.id
    );
    assert_eq!(
        last(frontend, "mode(.d3_value)"),
        strings(&[row.mode.unwrap()]),
        "{} mode",
        row.id
    );
    assert_eq!(
        last(frontend, "length(.d3_value)"),
        Sexp::Integer(vec![Some(i32::try_from(row.length.unwrap()).unwrap())]),
        "{} length",
        row.id
    );
    assert!(
        frontend.runtime_mut().take_output().is_empty(),
        "{} metadata output",
        row.id
    );
}
