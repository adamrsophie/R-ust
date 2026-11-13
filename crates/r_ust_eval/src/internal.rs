mod arity;
mod character;
mod conditions;
mod dispatch;
mod find_interval;
mod logical;
mod partial_match;
mod sequence;
mod strtoi;
mod unicode;
mod vector;

use character::character_values;
pub use dispatch::resolve;
pub(crate) use logical::as_logical;

use r_ust_sexp::InternalOperation;

use super::{CallArgument, EvalError, EvaluatedArgument, RuntimeState, Sexp};

impl RuntimeState {
    pub(super) fn eval_internal(
        &mut self,
        operation: InternalOperation,
        arguments: &[CallArgument],
    ) -> Result<Sexp, EvalError> {
        if operation == InternalOperation::EncodeStringWrapper {
            let value = self.encode_string_wrapper()?;
            self.visible = true;
            return Ok(value);
        }
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
            InternalOperation::RunLengthEncode => super::run_length::encode(args),
            InternalOperation::RunLengthDecode => self.inverse_rle(args),
            InternalOperation::RowsumValidate => self.call_rowsum(true, args),
            InternalOperation::RowsumAggregate => self.call_rowsum(false, args),
            InternalOperation::EncodeString => self.encode_string_values(
                &args.iter().map(|arg| arg.value.clone()).collect::<Vec<_>>(),
            ),
            InternalOperation::EncodeStringWrapper => self.encode_string_wrapper(),
            InternalOperation::Sequence => self.sequence(args),
            InternalOperation::Utf8ToInt => self.utf8_to_int(&args[0].value),
            InternalOperation::IntToUtf8 => self.int_to_utf8(args),
            InternalOperation::FindInterval => find_interval::find_interval(args),
            InternalOperation::Strtoi => strtoi::convert(&args[0].value, &args[1].value),
            InternalOperation::Pmatch => self.partial_match(args, false),
            InternalOperation::Charmatch => self.partial_match(args, true),
            InternalOperation::Vector => self.make_vector(&args[0].value, &args[1].value),
            InternalOperation::Stop => conditions::stop(args),
            InternalOperation::Inherits => conditions::inherits(args),
            InternalOperation::Gettext => conditions::gettext(args),
            InternalOperation::SignalCondition | InternalOperation::DefaultStop => {
                Err(EvalError::NotImplemented("condition-object signaling"))
            }
        }
    }
}
