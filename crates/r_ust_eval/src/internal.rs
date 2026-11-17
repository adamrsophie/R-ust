mod arity;
mod site;
mod stop;
pub(crate) use site::InternalSite;
mod bitwise;
mod character;
mod conditions;
mod deparse;
mod dispatch;
mod find_interval;
#[cfg(test)]
mod formatting_tests;
mod get;
mod invocation;
mod is_vector;
mod lapply;
mod logical;
mod partial_match;
mod sequence;
pub(crate) mod sprintf;
mod strtoi;
mod symbol;
mod translation;
mod unicode;
mod vector;
mod warning;

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
        if operation == InternalOperation::Lapply {
            self.internal_arity(operation, arguments.len(), site.inner)?;
            let result = self.internal_lapply(arguments, site.inner.unwrap_or(site.outer))?;
            self.visible = true;
            return Ok(result);
        }
        let args = self.eval_arguments(arguments, true, Some(site.outer))?;
        let value = self.call_internal(operation, &args, site.inner)?;
        if !matches!(
            operation,
            InternalOperation::Eval | InternalOperation::DoCall
        ) {
            self.visible = !matches!(
                operation,
                InternalOperation::AddConditionHandlers
                    | InternalOperation::ResetConditionHandlers
                    | InternalOperation::PrintDefault
                    | InternalOperation::Warning
                    | InternalOperation::Assign
                    | InternalOperation::Quit
            );
        }
        Ok(value)
    }

    pub(super) fn call_internal(
        &mut self,
        operation: InternalOperation,
        args: &[EvaluatedArgument],
        inner_call: Option<&Sexp>,
    ) -> Result<Sexp, EvalError> {
        self.internal_arity(operation, args.len(), inner_call)?;
        match operation {
            InternalOperation::Quit => self.internal_quit(args),
            InternalOperation::CommandArgs => Ok(Sexp::String(
                self.command_args.iter().cloned().map(Some).collect(),
            )),
            InternalOperation::Readline => self.internal_readline(&args[0].value),
            InternalOperation::Identical => {
                crate::identity::call(args, &self.source_current_call())
            }
            InternalOperation::Eval => self.internal_eval(args),
            InternalOperation::Assign => self.internal_assign(args),
            InternalOperation::Sprintf => self.internal_sprintf(args),
            InternalOperation::Deparse => self.internal_deparse(args),
            InternalOperation::Ngettext => self.internal_ngettext(args),
            InternalOperation::L10nInfo => self.internal_l10n_info(),
            InternalOperation::PrintDefault => self.print_default(args),
            InternalOperation::Environment => self.function_environment(&args[0].value),
            InternalOperation::NewEnvironment => self.new_environment(args),
            InternalOperation::ParentFrame => self.parent_frame(&args[0].value),
            InternalOperation::SysCall
            | InternalOperation::SysFrame
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
            InternalOperation::IsVector => is_vector::test(args, &self.source_current_call()),
            InternalOperation::AsVector => {
                if matches!(args[1].value.character_values(), Some([Some(mode)]) if mode == "symbol" || mode == "name")
                {
                    symbol::convert(&args[0].value).map_err(|error| match error {
                        EvalError::Domain(message) => {
                            crate::SourceFault::domain(message, &self.source_current_call())
                        }
                        EvalError::Type(message) => {
                            crate::SourceFault::type_error(message, &self.source_current_call())
                        }
                        other => other,
                    })
                } else {
                    crate::list_attributes::as_vector(args)
                }
            }
            InternalOperation::Warning => self.internal_warning(args),
            InternalOperation::Lapply => Err(EvalError::Control(
                "lapply requires unevaluated arguments".into(),
            )),
            InternalOperation::DoCall => self.internal_do_call(args),
            InternalOperation::Get => self.internal_get(args),
            InternalOperation::AddRestart => self.add_restart(args),
            InternalOperation::GetRestart => self.get_restart(args),
            InternalOperation::InvokeRestart => self.invoke_restart(args),
            InternalOperation::DefaultWarning => self.default_condition_warning(args),
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
