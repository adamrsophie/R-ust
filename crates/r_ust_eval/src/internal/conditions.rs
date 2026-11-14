use super::as_logical;
use crate::{class_value, EvalError, EvaluatedArgument, Sexp};

pub(super) fn inherits(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let (Sexp::String(what) | Sexp::AttributedCharacter { values: what, .. }) = &args[1].value
    else {
        return Err(EvalError::Domain(
            "'what' must be a character vector or an object with a nameOfClass() method".into(),
        ));
    };
    let Sexp::Logical(which) = &args[2].value else {
        return Err(EvalError::Domain(
            "'which' must be a length 1 logical vector".into(),
        ));
    };
    let [which] = which.as_slice() else {
        return Err(EvalError::Domain(
            "'which' must be a length 1 logical vector".into(),
        ));
    };
    let which = which.ok_or_else(|| EvalError::Domain("NA in coercion to boolean".into()))?;
    let (Sexp::String(classes)
    | Sexp::AttributedCharacter {
        values: classes, ..
    }) = class_value(&args[0].value)?
    else {
        return Err(EvalError::Type("invalid class representation".into()));
    };
    let positions = what
        .iter()
        .map(|name| {
            let position = classes
                .iter()
                .position(|class| class == name)
                .map_or(0, |index| index + 1);
            i32::try_from(position)
                .map(Some)
                .map_err(|_| EvalError::Domain("class index overflow".into()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(if which {
        Sexp::Integer(positions)
    } else {
        Sexp::Logical(vec![Some(
            positions.iter().any(|position| *position != Some(0)),
        )])
    })
}

pub(super) fn gettext(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    if crate::value_len(&args[1].value)? == 0 {
        return Ok(args[1].value.clone());
    }
    if !matches!(
        args[1].value,
        Sexp::String(_) | Sexp::AttributedCharacter { .. }
    ) {
        return Err(EvalError::Domain("invalid 'string' value".into()));
    }
    let translate = match &args[0].value {
        Sexp::Nil => true,
        Sexp::String(values) | Sexp::AttributedCharacter { values, .. } => match values.first() {
            Some(Some(domain)) => !domain.is_empty(),
            Some(None) => true,
            None => return Err(EvalError::Domain("invalid 'domain' value".into())),
        },
        Sexp::Logical(values) if values.as_slice() == [None] => false,
        Sexp::MissingArg(_)
        | Sexp::Symbol(_)
        | Sexp::MissingValue
        | Sexp::PairList(_)
        | Sexp::Logical(_)
        | Sexp::Integer(_)
        | Sexp::Real(_)
        | Sexp::Vector(_)
        | Sexp::ClassedList { .. }
        | Sexp::List(_)
        | Sexp::Factor { .. }
        | Sexp::DataFrame { .. }
        | Sexp::Matrix { .. }
        | Sexp::Dots(_)
        | Sexp::Promise { .. }
        | Sexp::Closure { .. }
        | Sexp::Char(_)
        | Sexp::ListObject(_)
        | Sexp::PairListObject(_)
        | Sexp::Environment(_)
        | Sexp::Primitive(_)
        | Sexp::InternalCall { .. }
        | Sexp::Function { .. }
        | Sexp::If { .. }
        | Sexp::For { .. }
        | Sexp::While { .. }
        | Sexp::Repeat { .. }
        | Sexp::Break
        | Sexp::Next
        | Sexp::Block(_)
        | Sexp::Subset { .. }
        | Sexp::Subset2D { .. }
        | Sexp::ExtractArguments { .. }
        | Sexp::Extract { .. }
        | Sexp::Dollar { .. }
        | Sexp::Call { .. }
        | Sexp::Assignment { .. }
        | Sexp::SuperAssignment { .. }
        | Sexp::Subassignment { .. }
        | Sexp::MatrixSubassignment { .. }
        | Sexp::ExtractAssignment { .. }
        | Sexp::DollarAssignment { .. }
        | Sexp::MemberAssignment { .. }
        | Sexp::ReplacementAssignment { .. }
        | Sexp::NamesAssignment { .. }
        | Sexp::Binary { .. }
        | Sexp::Unary { .. }
        | Sexp::Parenthesized(_) => return Err(EvalError::Domain("invalid 'domain' value".into())),
    };
    if let (true, Some(trim)) = (translate, args.get(2)) {
        let trim = &trim.value;
        if matches!(
            trim,
            Sexp::Logical(_)
                | Sexp::Integer(_)
                | Sexp::Real(_)
                | Sexp::String(_)
                | Sexp::AttributedCharacter { .. }
                | Sexp::Matrix { .. }
                | Sexp::Factor { .. }
        ) && crate::value_len(trim)? > 1
        {
            return Err(EvalError::Domain(format!(
                "'length = {}' in coercion to 'logical(1)'",
                crate::value_len(trim)?
            )));
        }
        as_logical(trim).ok_or_else(|| EvalError::Domain("NA in coercion to boolean".into()))?;
    }
    // shortcut: untranslated C-locale text only, add catalogues with locale support.
    Ok(args[1].value.clone())
}
