mod names;
mod options;
mod storage;
#[cfg(test)]
mod tests;

use crate::{EvalError, EvaluatedArgument, Sexp};

pub(super) fn concatenate(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let options = options::extract(args)?;
    let args = options
        .arguments
        .iter()
        .map(|arg| {
            Ok(EvaluatedArgument {
                name: arg.name.clone(),
                value: crate::ordinary_values::read(&arg.value)?.into_owned(),
            })
        })
        .collect::<Result<Vec<_>, EvalError>>()?;
    if args.iter().any(|arg| {
        matches!(
            arg.value,
            Sexp::ClassedList { .. } | Sexp::NamedList { .. } | Sexp::List(_) | Sexp::PairList(_)
        )
    }) {
        return legacy_lists(args, options.recursive, options.use_names);
    }
    let result = storage::concatenate(&args)?;
    if options.use_names {
        if let Some(names) = names::extract(&args)? {
            return crate::list_attributes::engine::set(&result, "names", &Sexp::String(names));
        }
    }
    Ok(result)
}

fn legacy_lists(
    mut args: Vec<EvaluatedArgument>,
    recursive: bool,
    use_names: bool,
) -> Result<Sexp, EvalError> {
    for (name, value) in [("recursive", recursive), ("use.names", use_names)] {
        args.push(EvaluatedArgument {
            name: Some(name.into()),
            value: Sexp::Logical(vec![Some(value)]),
        });
    }
    crate::list_concatenation::concatenate(&args)
}
