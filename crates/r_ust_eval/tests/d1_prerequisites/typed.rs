#[path = "../dynamic_frames/compare.rs"]
mod compare;
#[path = "../dynamic_frames/double.rs"]
mod double;
#[path = "../support/fixture_hex.rs"]
mod json;
#[path = "../dynamic_frames/node.rs"]
mod node;

use r_ust_eval::{EvalError, Frontend};
use r_ust_sexp::Sexp;
use std::collections::BTreeSet;

const FIXTURE: &str = include_str!("../fixtures/d1-prerequisites-r-4.6.0.tsv");
const FOUNDATION: [&str; 10] = [
    "class_names_type",
    "attribute_creation_order",
    "attribute_remove_order",
    "attribute_readd_order",
    "class_message",
    "class_extract",
    "pairlist_tagged",
    "pairlist_empty",
    "pairlist_conversion",
    "null_attribute",
];

#[test]
fn independent_typed_foundation_reference() {
    let rows = read(FIXTURE).unwrap();
    let mut checked = BTreeSet::new();
    for row in rows {
        if !FOUNDATION.contains(&row.id.as_str()) {
            continue;
        }
        let mut frontend = Frontend::default();
        if !row.setup.is_empty() {
            frontend.eval_source(&row.setup).unwrap();
        }
        let result = frontend.eval_source(&row.expression);
        match result {
            Ok(values) => {
                assert_eq!(row.status, "0", "{}", row.id);
                let [value] = values.as_slice() else {
                    panic!("{} result count", row.id);
                };
                let (visible, expected) = node::parse(&row.node).unwrap();
                assert_eq!(
                    frontend.runtime().is_visible(),
                    visible,
                    "{} visibility",
                    row.id
                );
                compare::compare(&expected, value)
                    .unwrap_or_else(|error| panic!("{}: {error}", row.id));
            }
            Err(error) => {
                assert_eq!(row.status, "1", "{}: {error}", row.id);
                let EvalError::Stop {
                    message,
                    call_requested: true,
                } = error
                else {
                    panic!("{} wrong condition", row.id);
                };
                assert_eq!(row.frame, "stop");
                let (kind, payload) = row.conditions.trim_end().split_once('\t').unwrap();
                assert_eq!(kind, "error");
                assert_eq!(message, json::unhex(payload).unwrap());
            }
        }
        assert!(
            frontend.runtime_mut().take_output().is_empty(),
            "{} unexpected effects",
            row.id
        );
        assert_eq!(
            frontend.eval_source("sys.nframe()").unwrap(),
            [Sexp::Integer(vec![Some(0)])]
        );
        println!("D1_FOUNDATION_TYPED\t{}\tPASS", row.id);
        checked.insert(row.id);
    }
    assert_eq!(checked, FOUNDATION.into_iter().map(String::from).collect());
}

struct Row {
    id: String,
    setup: String,
    expression: String,
    node: String,
    status: String,
    conditions: String,
    frame: String,
}

fn read(text: &str) -> Result<Vec<Row>, String> {
    if !text.ends_with('\n') {
        return Err("fixture final newline".into());
    }
    let mut lines = text.lines();
    if lines.next()
        != Some("id\tsetup_hex\texpression_hex\tnode_hex\tstatus\tconditions_hex\tframe")
    {
        return Err("fixture header".into());
    }
    let mut seen = BTreeSet::new();
    let mut rows = Vec::new();
    for line in lines {
        let fields: Vec<_> = line.split('\t').collect();
        let [id, setup, expression, encoded, status, conditions, frame] = fields.as_slice() else {
            return Err("fixture fields".into());
        };
        if id.is_empty()
            || !id
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte == b'_')
            || !seen.insert(*id)
        {
            return Err("fixture id".into());
        }
        let node = json::unhex(encoded)?;
        let conditions = json::unhex(conditions)?;
        if !matches!(*status, "0" | "1")
            || !matches!(*frame, "none" | "domain" | "arity" | "type" | "stop")
        {
            return Err("fixture status".into());
        }
        if *status == "0" {
            node::parse(&node)?;
        } else if !node.is_empty() {
            return Err("error with value".into());
        }
        for condition in conditions.lines() {
            let (kind, payload) = condition.split_once('\t').ok_or("condition fields")?;
            if !matches!(kind, "error" | "warning") {
                return Err("condition kind".into());
            }
            json::unhex(payload)?;
        }
        rows.push(Row {
            id: (*id).into(),
            setup: json::unhex(setup)?,
            expression: json::unhex(expression)?,
            node,
            status: (*status).into(),
            conditions,
            frame: (*frame).into(),
        });
    }
    if rows.len() != 33 {
        return Err("fixture membership".into());
    }
    Ok(rows)
}

#[test]
fn prerequisite_reader_rejects_corruption() {
    assert!(read(FIXTURE).is_ok());
    for corrupted in [
        FIXTURE.replacen("class_names_type", "class_message", 1),
        FIXTURE.replacen("\t0\t", "\t2\t", 1),
        FIXTURE.replacen("44464e4f444531", "44464e4f444532", 1),
        FIXTURE.trim_end().to_string(),
    ] {
        assert!(read(&corrupted).is_err());
    }
}
