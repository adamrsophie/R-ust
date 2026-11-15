// R 4.6.0 src/main/identical.c: Copyright (C) 2001-2025 The R Core Team
use crate::{EvalError, EvaluatedArgument, Sexp, SourceFault};

const NAMES: [&str; 7] = [
    "num.eq",
    "single.NA",
    "attrib.as.set",
    "ignore.bytecode",
    "ignore.environment",
    "ignore.srcref",
    "extptr.as.ref",
];

pub(super) struct Inputs<'a> {
    pub(super) left: &'a Sexp,
    pub(super) right: &'a Sexp,
    pub(super) flags: [bool; 7],
}

pub(super) fn read<'a>(
    args: &'a [EvaluatedArgument],
    call: &Sexp,
) -> Result<Inputs<'a>, EvalError> {
    if args.len() < 5 {
        return Err(SourceFault::arity(
            format!(
                "{} arguments passed to .Internal(identical) which requires 8",
                args.len()
            ),
            call,
        ));
    }
    let mut flags = [true, true, true, true, false, true, false];
    for (index, argument) in args.iter().skip(2).take(7).enumerate() {
        flags[index] = crate::internal::as_logical(&argument.value).ok_or_else(|| {
            SourceFault::domain(format!("invalid '{}' value", NAMES[index]), call)
        })?;
    }
    Ok(Inputs {
        left: &args[0].value,
        right: &args[1].value,
        flags,
    })
}
