use crate::{EvalError, EvaluatedArgument, Sexp};

pub(super) struct Options {
    pub(super) arguments: Vec<EvaluatedArgument>,
    pub(super) recursive: bool,
    pub(super) use_names: bool,
}

pub(super) fn extract(args: &[EvaluatedArgument]) -> Result<Options, EvalError> {
    let mut result = Options {
        arguments: Vec::new(),
        recursive: false,
        use_names: true,
    };
    let mut seen = [false; 2];
    for (index, arg) in args.iter().enumerate() {
        let value = crate::ordinary_values::read(&arg.value)?;
        // do_c compacts NULL arguments before c_Extract_opt, retaining the first.
        if index != 0 && matches!(value.as_ref(), Sexp::Nil) {
            continue;
        }
        let (slot, target) = match arg.name.as_deref() {
            Some("recursive") => (0, &mut result.recursive),
            Some("use.names") => (1, &mut result.use_names),
            _ => {
                result.arguments.push(arg.clone());
                continue;
            }
        };
        if seen[slot] {
            let name = if slot == 0 { "recursive" } else { "use.names" };
            return Err(EvalError::Domain(format!(
                "repeated formal argument '{name}'"
            )));
        }
        seen[slot] = true;
        let value = crate::coercion::storage::read(value.as_ref())?;
        if let Some(flag) = crate::internal::as_logical(value.as_ref()) {
            *target = flag;
        }
    }
    Ok(result)
}
