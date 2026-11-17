use super::stop;
use crate::{EvalError, EvaluatedArgument, Sexp};
use r_ust_sexp::{Environment, ValueImport, ValueObject};

#[test]
fn managed_stop_messages_and_call_flags_preserve_condition_envelopes() -> Result<(), EvalError> {
    let owner = Environment::global();
    let flag = Sexp::Logical(vec![Some(false)]);
    let managed_flag = Sexp::Managed(ValueObject::import(&owner, &flag, ValueImport::Evaluated)?);
    for message in [Sexp::Nil, Sexp::String(vec![Some("kept".into())])] {
        let managed_message = Sexp::Managed(ValueObject::import(
            &owner,
            &message,
            ValueImport::Evaluated,
        )?);
        let arguments = |flag, message| {
            [
                EvaluatedArgument {
                    name: None,
                    value: flag,
                },
                EvaluatedArgument {
                    name: None,
                    value: message,
                },
            ]
        };
        assert_eq!(
            stop(&arguments(flag.clone(), message), &Sexp::Nil),
            stop(
                &arguments(managed_flag.clone(), managed_message),
                &Sexp::Nil
            )
        );
    }
    Ok(())
}
