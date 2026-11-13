mod arity;
mod character;
mod dispatch;
mod find_interval;
mod logical;
mod partial_match;
mod strtoi;
mod unicode;
mod vector;

use character::character_values;
pub use dispatch::resolve;
use logical::as_logical;

use r_ust_sexp::InternalOperation;

use super::{class_value, CallArgument, EvalError, EvaluatedArgument, RuntimeState, Sexp};

impl RuntimeState {
    pub(super) fn eval_internal(
        &mut self,
        operation: InternalOperation,
        arguments: &[CallArgument],
    ) -> Result<Sexp, EvalError> {
        let args = self.eval_arguments(arguments, true)?;
        let value = self.call_internal(operation, &args)?;
        self.visible = true;
        Ok(value)
    }

    fn call_internal(
        &mut self,
        operation: InternalOperation,
        args: &[EvaluatedArgument],
    ) -> Result<Sexp, EvalError> {
        arity::check(operation, args.len())?;
        match operation {
            InternalOperation::Utf8ToInt => self.utf8_to_int(&args[0].value),
            InternalOperation::IntToUtf8 => self.int_to_utf8(args),
            InternalOperation::FindInterval => find_interval::find_interval(args),
            InternalOperation::Strtoi => strtoi::convert(&args[0].value, &args[1].value),
            InternalOperation::Pmatch => self.partial_match(args, false),
            InternalOperation::Charmatch => self.partial_match(args, true),
            InternalOperation::Vector => self.make_vector(&args[0].value, &args[1].value),
            InternalOperation::Stop => {
                let call_requested = as_logical(&args[0].value).unwrap_or(true);
                let message = match &args[1].value {
                    Sexp::Nil => String::new(),
                    value @ (Sexp::MissingArg(_)
                    | Sexp::Symbol(_)
                    | Sexp::Logical(_)
                    | Sexp::Integer(_)
                    | Sexp::Real(_)
                    | Sexp::String(_)
                    | Sexp::Vector(_)
                    | Sexp::List(_)
                    | Sexp::Factor { .. }
                    | Sexp::DataFrame { .. }
                    | Sexp::Matrix { .. }
                    | Sexp::Dots(_)
                    | Sexp::Promise { .. }
                    | Sexp::Closure { .. }
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
                    | Sexp::Extract { .. }
                    | Sexp::Dollar { .. }
                    | Sexp::Call { .. }
                    | Sexp::Assignment { .. }
                    | Sexp::SuperAssignment { .. }
                    | Sexp::Subassignment { .. }
                    | Sexp::MatrixSubassignment { .. }
                    | Sexp::ExtractAssignment { .. }
                    | Sexp::DollarAssignment { .. }
                    | Sexp::NamesAssignment { .. }
                    | Sexp::Binary { .. }
                    | Sexp::Unary { .. }
                    | Sexp::Parenthesized(_)) => character_values(value)?
                        .first()
                        .cloned()
                        .ok_or_else(|| EvalError::Domain(" [invalid string in stop(.)]".into()))?
                        .unwrap_or_else(|| "NA".into()),
                };
                Err(EvalError::Stop {
                    message,
                    call_requested,
                })
            }
            InternalOperation::Inherits => {
                let Sexp::String(what) = &args[1].value else {
                    return Err(EvalError::Domain("'what' must be a character vector or an object with a nameOfClass() method".into()));
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
                let which =
                    which.ok_or_else(|| EvalError::Domain("NA in coercion to boolean".into()))?;
                let Sexp::String(classes) = class_value(&args[0].value) else {
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
            InternalOperation::Gettext => {
                if super::value_len(&args[1].value) == 0 {
                    return Ok(args[1].value.clone());
                }
                if !matches!(args[1].value, Sexp::String(_)) {
                    return Err(EvalError::Domain("invalid 'string' value".into()));
                }
                let translate = match &args[0].value {
                    Sexp::Nil => true,
                    Sexp::String(values) => match values.first() {
                        Some(Some(domain)) => !domain.is_empty(),
                        Some(None) => true,
                        None => return Err(EvalError::Domain("invalid 'domain' value".into())),
                    },
                    Sexp::Logical(values) if values.as_slice() == [None] => false,
                    Sexp::MissingArg(_)
                    | Sexp::Symbol(_)
                    | Sexp::Logical(_)
                    | Sexp::Integer(_)
                    | Sexp::Real(_)
                    | Sexp::Vector(_)
                    | Sexp::List(_)
                    | Sexp::Factor { .. }
                    | Sexp::DataFrame { .. }
                    | Sexp::Matrix { .. }
                    | Sexp::Dots(_)
                    | Sexp::Promise { .. }
                    | Sexp::Closure { .. }
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
                    | Sexp::Extract { .. }
                    | Sexp::Dollar { .. }
                    | Sexp::Call { .. }
                    | Sexp::Assignment { .. }
                    | Sexp::SuperAssignment { .. }
                    | Sexp::Subassignment { .. }
                    | Sexp::MatrixSubassignment { .. }
                    | Sexp::ExtractAssignment { .. }
                    | Sexp::DollarAssignment { .. }
                    | Sexp::NamesAssignment { .. }
                    | Sexp::Binary { .. }
                    | Sexp::Unary { .. }
                    | Sexp::Parenthesized(_) => {
                        return Err(EvalError::Domain("invalid 'domain' value".into()))
                    }
                };
                if let (true, Some(trim)) = (translate, args.get(2)) {
                    let trim = &trim.value;
                    if matches!(
                        trim,
                        Sexp::Logical(_)
                            | Sexp::Integer(_)
                            | Sexp::Real(_)
                            | Sexp::String(_)
                            | Sexp::Matrix { .. }
                            | Sexp::Factor { .. }
                    ) && super::value_len(trim) > 1
                    {
                        return Err(EvalError::Domain(format!(
                            "'length = {}' in coercion to 'logical(1)'",
                            super::value_len(trim)
                        )));
                    }
                    as_logical(trim)
                        .ok_or_else(|| EvalError::Domain("NA in coercion to boolean".into()))?;
                }
                // shortcut: untranslated C-locale text only, add catalogues with locale support.
                Ok(args[1].value.clone())
            }
            InternalOperation::SignalCondition | InternalOperation::DefaultStop => {
                Err(EvalError::NotImplemented("condition-object signaling"))
            }
        }
    }
}
