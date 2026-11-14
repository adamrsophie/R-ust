mod arity;
mod site;
mod stop;
pub(crate) use site::InternalSite;
mod bitwise;
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
        site: InternalSite<'_>,
    ) -> Result<Sexp, EvalError> {
        if operation == InternalOperation::EncodeStringWrapper {
            let value = self.encode_string_wrapper()?;
            self.visible = true;
            return Ok(value);
        }
        let args = self.eval_arguments(arguments, true, Some(site.outer))?;
        let value = self.call_internal(operation, &args, site.inner)?;
        self.visible = !matches!(
            operation,
            InternalOperation::AddConditionHandlers
                | InternalOperation::ResetConditionHandlers
                | InternalOperation::PrintDefault
        );
        Ok(value)
    }

    fn call_internal(
        &mut self,
        operation: InternalOperation,
        args: &[EvaluatedArgument],
        inner_call: Option<&Sexp>,
    ) -> Result<Sexp, EvalError> {
        self.internal_arity(operation, args.len(), inner_call)?;
        match operation {
            InternalOperation::PrintDefault => self.print_default(args),
            InternalOperation::Environment => self.function_environment(&args[0].value),
            InternalOperation::NewEnvironment => self.new_environment(args),
            InternalOperation::ParentFrame => self.parent_frame(&args[0].value),
            InternalOperation::SysFrame
            | InternalOperation::SysFunction
            | InternalOperation::SysParent
            | InternalOperation::SysNFrame => self.dynamic_frame(operation, args),
            InternalOperation::ParentEnvironment => {
                crate::environment_values::parent(&args[0].value, &self.source_current_call())
            }
            InternalOperation::EnvironmentName => crate::environment_values::name(&args[0].value),
            InternalOperation::Formals => {
                self.formals(&args[0].value, inner_call.unwrap_or(&Sexp::Nil))
            }
            InternalOperation::Body => self.body(&args[0].value, inner_call.unwrap_or(&Sexp::Nil)),
            InternalOperation::BitwiseNot => self.bitwise_not(&args[0].value),
            InternalOperation::BitwiseAnd => self.bitwise_binary(bitwise::Binary::And, args),
            InternalOperation::BitwiseOr => self.bitwise_binary(bitwise::Binary::Or, args),
            InternalOperation::BitwiseXor => self.bitwise_binary(bitwise::Binary::Xor, args),
            InternalOperation::BitwiseShiftL => self.bitwise_binary(bitwise::Binary::ShiftL, args),
            InternalOperation::BitwiseShiftR => self.bitwise_binary(bitwise::Binary::ShiftR, args),
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
            InternalOperation::AsVector => crate::list_attributes::as_vector(args),
            InternalOperation::Vector => self.make_vector(&args[0].value, &args[1].value),
            InternalOperation::Stop => stop::stop(
                args,
                &self.source_stop_call(as_logical(&args[0].value).unwrap_or(true)),
            ),
            InternalOperation::Inherits => conditions::inherits(args),
            InternalOperation::Gettext => conditions::gettext(args),
            InternalOperation::SignalCondition => self.signal_condition(args),
            InternalOperation::DefaultStop => self.default_condition_stop(args),
            InternalOperation::AddConditionHandlers => self.add_condition_handlers(args),
            InternalOperation::ResetConditionHandlers => self.reset_condition_handlers(args),
            InternalOperation::GetErrorMessage => {
                Ok(Sexp::String(vec![Some(self.conditions.message.clone())]))
            }
        }
    }
}
