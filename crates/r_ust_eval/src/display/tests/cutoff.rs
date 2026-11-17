use super::{
    support::expected,
    values::{a, i, s},
};
use crate::language_print::{format_deparse, Profile};
use crate::{EvalError, RuntimeState, Sexp};
use r_ust_sexp::{AtomicVector, Environment, ValueImport, ValueObject};

#[test]
fn managed_print_deparse_attribute_cutoff() -> Result<(), EvalError> {
    let raw = Sexp::AttributedAtomic {
        values: AtomicVector::Integer(vec![Some(1)]),
        attributes: vec![
            a("z", i(&[2])),
            a("a", s(&[Some("x")])),
            a("comment", s(&[Some("hidden")])),
            a("srcref", i(&[1])),
            a("wholeSrcref", i(&[2])),
            a("srcfile", s(&[Some("hidden")])),
        ],
    };
    let owner = Environment::default();
    let object = ValueObject::import(&owner, &raw, ValueImport::Evaluated)?;
    for value in [&raw, &object.transport()?] {
        let (lines, incomplete) = format_deparse(
            &mut RuntimeState::default(),
            value,
            Profile {
                cutoff: 20,
                backtick: false,
                control: 1093,
                lines: Some(2),
                utf8: true,
            },
        )?;
        assert!(!incomplete);
        assert_eq!(
            format!("{}\n", lines.join("\n")),
            expected("deparse_cutoff")
        );
    }
    Ok(())
}
