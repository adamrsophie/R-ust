use super::{
    compare,
    conditions::{self, Conditions},
    node,
    reader::{self, Row},
};
use r_ust_eval::Frontend;
use r_ust_sexp::{AssignmentOperator, ClosureEnvironment, FunctionParameter, Sexp};
use std::collections::BTreeSet;

const ORIGINAL: &str = include_str!("../fixtures/dynamic-frames-r-4.6.0.tsv");
const SUPPLEMENTAL: &str = include_str!("../fixtures/dynamic-frame-source-r-4.6.0.tsv");

pub(crate) fn run_original() {
    let rows = reader::read(ORIGINAL, 184).unwrap();
    let expected = conditions::read().unwrap();
    let mut references = BTreeSet::new();
    let mut failures = Vec::new();
    let mut checked = 0;
    for row in rows {
        assert_eq!(row.contract, "supported");
        let source = format!(
            "{}{}\n",
            if row.setup.is_empty() {
                String::new()
            } else {
                format!("{}\n", row.setup)
            },
            row.expression
        );
        assert_eq!(row.source, source, "{} source", row.id);
        if row.scope == "sdk-reference-only" {
            references.insert(row.id);
            continue;
        }
        let result = check(&row, expected.get(&row.id).unwrap());
        println!(
            "D1_TYPED\t{}\t{}",
            row.id,
            if result.is_ok() { "PASS" } else { "FAIL" }
        );
        if let Err(error) = result {
            failures.push(format!("{}: {error}", row.id));
        }
        checked += 1;
    }
    assert_eq!(
        references,
        [
            "sdk_evalq_same_environment",
            "sdk_evalq_break_same_environment",
            "sdk_evalq_next_same_environment",
            "sdk_evalq_return_boundary"
        ]
        .map(String::from)
        .into_iter()
        .collect()
    );
    assert_eq!(checked, 180);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    println!("D1_ACCOUNTED original=184 typed_native=180 private_context_reference=4");
}

pub(crate) fn run_supplemental() {
    let rows = reader::read(SUPPLEMENTAL, 30).unwrap();
    let expected = conditions::read().unwrap();
    let mut pending = BTreeSet::new();
    let mut checked = 0;
    let mut sdk = 0;
    let mut failures = Vec::new();
    for row in rows {
        if row.contract == "pending-bytecode" {
            pending.insert(row.id);
            continue;
        }
        if row.scope == "sdk-reference-only" {
            assert_eq!(row.id, "closure_namespace_sdk");
            sdk += 1;
        }
        let result = check(&row, expected.get(&row.id).unwrap());
        println!(
            "D1_SOURCE_TYPED\t{}\t{}",
            row.id,
            if result.is_ok() { "PASS" } else { "FAIL" }
        );
        if let Err(error) = result {
            failures.push(format!("{}: {error}", row.id));
        }
        checked += 1;
    }
    assert_eq!(checked, 27);
    assert_eq!(sdk, 1);
    assert_eq!(
        pending,
        ["closure_identity", "closure_force", "closure_base"]
            .map(String::from)
            .into_iter()
            .collect()
    );
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    println!("D1_ACCOUNTED supplemental=30 typed_cli=26 typed_sdk=1 pending_bytecode=3");
}

fn setup(row: &Row) -> Result<Frontend, String> {
    let mut frontend = Frontend::default();
    if row.id == "closure_namespace_sdk" {
        let namespace = frontend
            .eval_source(".BaseNamespaceEnv")
            .map_err(|error| error.to_string())?;
        let [Sexp::Environment(namespace)] = namespace.as_slice() else {
            return Err("SDK namespace".into());
        };
        let closure = Sexp::Closure {
            parameters: vec![FunctionParameter {
                name: "x".into(),
                default: None,
            }],
            body: Box::new(Sexp::Symbol("x".into())),
            environment: ClosureEnvironment::Captured(namespace.clone()),
        };
        frontend
            .runtime_mut()
            .eval(&Sexp::Assignment {
                operator: AssignmentOperator::Left,
                target: "f".into(),
                value: Box::new(closure),
            })
            .map_err(|error| error.to_string())?;
    } else if !row.setup.is_empty() {
        frontend
            .eval_source_printed(&row.setup)
            .map_err(|error| format!("setup: {error:?}"))?;
    }
    Ok(frontend)
}

fn check(row: &Row, conditions: &Conditions) -> Result<(), String> {
    let mut frontend = setup(row)?;
    let result = frontend.eval_source(&row.expression);
    match result {
        Ok(values) => {
            if row.status != 0 {
                return Err(format!("expected terminal condition, got {values:?}"));
            }
            let [value] = values.as_slice() else {
                return Err("subject result count".into());
            };
            let (visible, expected) = node::parse(&row.node)?;
            if visible != frontend.runtime().is_visible() {
                return Err("visibility mismatch".into());
            }
            compare::compare(&expected, value)?;
        }
        Err(error) => {
            if row.status == 0 {
                return Err(format!("unexpected native error: {error:?}"));
            }
            conditions.check_error(&error)?;
        }
    }
    conditions.check_effects(frontend.runtime_mut().take_output(), reader::effects(row)?)?;
    let recovery = frontend
        .eval_source("sys.nframe();42L")
        .map_err(|error| format!("recovery: {error:?}"))?;
    if recovery != [Sexp::Integer(vec![Some(0)]), Sexp::Integer(vec![Some(42)])] {
        return Err("active contexts leaked after subject".into());
    }
    if !frontend.runtime_mut().take_output().is_empty() {
        return Err("effects leaked after subject".into());
    }
    Ok(())
}
