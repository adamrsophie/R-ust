use crate::language_print::{format_deparse, Profile};
use crate::{EvalError, RuntimeState, Sexp};
use r_ust_sexp::{Environment, ValueImport, ValueObject};

pub(super) fn expected(name: &str) -> &str {
    include_str!("reference.txt")
        .split(&format!("CASE {name}\n"))
        .nth(1)
        .expect("reference case")
        .split("\nEND\n")
        .next()
        .expect("reference end")
}

pub(super) fn deparse(value: &Sexp, control: u32, backtick: bool) -> Result<String, EvalError> {
    let mut runtime = RuntimeState::default();
    let (lines, incomplete) = format_deparse(
        &mut runtime,
        value,
        Profile {
            cutoff: 60,
            backtick,
            control,
            lines: None,
            utf8: true,
        },
    )?;
    assert!(!incomplete);
    assert!(runtime.take_output().is_empty());
    Ok(format!("{}\n", lines.join("\n")))
}

pub(super) fn check(name: &str, raw: Sexp) -> Result<(), EvalError> {
    let owner = Environment::default();
    let managed = ValueObject::import(&owner, &raw, ValueImport::Evaluated)?;
    let before = managed.view()?;
    let history = managed.semantic_reference_count()?;
    let descriptor = managed.borrow()?;
    let transport = managed.transport()?;
    drop(owner);
    for value in [&raw, &transport, descriptor.value()] {
        assert_eq!(
            super::super::format_sexp(value)?,
            expected(&format!("print_{name}")),
            "print {name}: {value:?}"
        );
        assert_eq!(
            super::super::character::format(value, true)?,
            expected(&format!("print_{name}")),
            "print.default {name}"
        );
        let backtick = matches!(
            &raw,
            Sexp::AttributedAggregate {
                values: r_ust_sexp::AggregateVector::Expression(_),
                ..
            }
        );
        assert_eq!(
            deparse(value, 1093, backtick)?,
            expected(&format!("deparse_{name}")),
            "deparse {name}"
        );
    }
    let after = managed.view()?;
    assert_eq!(before.header, after.header);
    assert_eq!(before.children.len(), after.children.len());
    for (a, b) in before.children.iter().zip(&after.children) {
        assert!(a.same_object(b));
    }
    assert_eq!(managed.semantic_reference_count()?, history);
    Ok(())
}

pub(super) fn layout(name: &str, raw: Sexp) -> Result<(), EvalError> {
    let expected = include_str!("layout_reference.txt")
        .split(&format!("CASE {name}\n"))
        .nth(1)
        .expect("layout reference")
        .split("\nEND\n")
        .next()
        .expect("layout reference end");
    let owner = Environment::default();
    let object = ValueObject::import(&owner, &raw, ValueImport::Evaluated)?;
    for value in [&raw, &object.transport()?] {
        assert_eq!(super::super::format_sexp(value)?, expected, "layout {name}");
    }
    Ok(())
}
