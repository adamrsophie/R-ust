use super::{atomic, strings::BlankPolicy};
use crate::{EvalError, RuntimeState, Sexp};

impl RuntimeState {
    pub(crate) fn vector_length(&mut self, value: &Sexp) -> Result<usize, EvalError> {
        let mut value = value;
        while let Sexp::Matrix { data, .. } = value {
            value = data;
        }
        let invalid = || EvalError::Domain("invalid 'length' argument".into());
        if crate::value_len(value) != 1 {
            return Err(invalid());
        }
        match value {
            Sexp::Integer(values) => {
                let length = values[0]
                    .ok_or_else(|| EvalError::Domain("vector size cannot be NA".into()))?;
                return usize::try_from(length).map_err(|_| invalid());
            }
            Sexp::Factor { codes, .. } => {
                return codes[0]
                    .ok_or_else(|| EvalError::Domain("vector size cannot be NA".into()));
            }
            Sexp::Real(_) | Sexp::String(_) => {}
            Sexp::Nil
            | Sexp::MissingArg(_)
            | Sexp::Symbol(_)
            | Sexp::MissingValue
            | Sexp::PairList(_)
            | Sexp::Logical(_)
            | Sexp::Vector(_)
            | Sexp::List(_)
            | Sexp::DataFrame { .. }
            | Sexp::Matrix { .. }
            | Sexp::Dots(_)
            | Sexp::Promise { .. }
            | Sexp::Closure { .. }
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
            | Sexp::NamesAssignment { .. }
            | Sexp::Binary { .. }
            | Sexp::Unary { .. }
            | Sexp::Parenthesized(_) => return Err(invalid()),
        }
        let (length, warnings) = atomic::scalar_real(value, BlankPolicy::environment());
        self.record_parse_warnings(&warnings.messages())?;
        if length.is_nan() {
            return Err(EvalError::Domain("vector size cannot be NA/NaN".into()));
        }
        if !length.is_finite() {
            return Err(EvalError::Domain("vector size cannot be infinite".into()));
        }
        if length > 4_503_599_627_370_496.0 {
            return Err(EvalError::Domain(
                "vector size specified is too large".into(),
            ));
        }
        if length.trunc() < 0.0 {
            return Err(invalid());
        }
        length
            .trunc()
            .max(0.0)
            .to_string()
            .parse()
            .map_err(|_| invalid())
    }
}
