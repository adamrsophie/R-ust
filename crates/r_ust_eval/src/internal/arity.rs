use crate::EvalError;
use r_ust_sexp::InternalOperation;

pub(super) fn check(
    operation: InternalOperation,
    count: usize,
    call: Option<&crate::Sexp>,
) -> Result<(), EvalError> {
    let (name, arity) = match operation {
        InternalOperation::Environment => ("environment", 1),
        InternalOperation::PrintDefault => ("print.default", 3),
        InternalOperation::NewEnvironment => ("new.env", 3),
        InternalOperation::ParentFrame => ("parent.frame", 1),
        InternalOperation::SysFrame
        | InternalOperation::SysFunction
        | InternalOperation::SysParent
        | InternalOperation::SysNFrame => return Ok(()),
        InternalOperation::ParentEnvironment => ("parent.env", 1),
        InternalOperation::EnvironmentName => ("environmentName", 1),
        InternalOperation::Formals => ("formals", 1),
        InternalOperation::Body => ("body", 1),
        InternalOperation::BitwiseNot => ("bitwiseNot", 1),
        InternalOperation::BitwiseAnd => ("bitwiseAnd", 2),
        InternalOperation::BitwiseOr => ("bitwiseOr", 2),
        InternalOperation::BitwiseXor => ("bitwiseXor", 2),
        InternalOperation::BitwiseShiftL => ("bitwiseShiftL", 2),
        InternalOperation::BitwiseShiftR => ("bitwiseShiftR", 2),
        InternalOperation::RunLengthEncode => ("rle", 1),
        InternalOperation::RunLengthDecode => ("inverse.rle", 1),
        InternalOperation::RowsumValidate => ("rowsum validation", 2),
        InternalOperation::RowsumAggregate => ("rowsum aggregation", 4),
        InternalOperation::EncodeString => ("encodeString", 5),
        InternalOperation::EncodeStringWrapper => ("encodeString wrapper", 0),
        InternalOperation::Sequence => ("sequence", 4),
        InternalOperation::Utf8ToInt => ("utf8ToInt", 1),
        InternalOperation::IntToUtf8 => ("intToUtf8", 3),
        InternalOperation::FindInterval => ("findInterval", 6),
        InternalOperation::Strtoi => ("strtoi", 2),
        InternalOperation::Pmatch => ("pmatch", 4),
        InternalOperation::Charmatch => ("charmatch", 3),
        InternalOperation::AsVector => ("as.vector", 2),
        InternalOperation::Vector => ("vector", 2),
        InternalOperation::Stop => ("stop", 2),
        InternalOperation::DefaultStop => (".dfltStop", 2),
        InternalOperation::AddConditionHandlers => (".addCondHands", 5),
        InternalOperation::ResetConditionHandlers => (".resetCondHands", 1),
        InternalOperation::GetErrorMessage => ("geterrmessage", 0),
        InternalOperation::Inherits => ("inherits", 3),
        InternalOperation::Gettext => ("gettext", 3),
        InternalOperation::SignalCondition => (".signalCondition", 3),
    };
    let valid_arity = match operation {
        InternalOperation::SysFrame
        | InternalOperation::SysFunction
        | InternalOperation::SysParent
        | InternalOperation::SysNFrame => true,
        InternalOperation::Gettext => {
            if !(2..=3).contains(&count) {
                return Err(arity_error(
                    "either 2 or 3 arguments are required".into(),
                    call,
                ));
            }
            true
        }
        InternalOperation::Environment
        | InternalOperation::PrintDefault
        | InternalOperation::NewEnvironment
        | InternalOperation::ParentFrame
        | InternalOperation::BitwiseNot
        | InternalOperation::ParentEnvironment
        | InternalOperation::EnvironmentName
        | InternalOperation::Formals
        | InternalOperation::Body
        | InternalOperation::BitwiseAnd
        | InternalOperation::BitwiseOr
        | InternalOperation::BitwiseXor
        | InternalOperation::BitwiseShiftL
        | InternalOperation::BitwiseShiftR
        | InternalOperation::RunLengthEncode
        | InternalOperation::RunLengthDecode
        | InternalOperation::RowsumValidate
        | InternalOperation::RowsumAggregate
        | InternalOperation::EncodeString
        | InternalOperation::EncodeStringWrapper
        | InternalOperation::Sequence
        | InternalOperation::AsVector
        | InternalOperation::Vector
        | InternalOperation::Utf8ToInt
        | InternalOperation::IntToUtf8
        | InternalOperation::FindInterval
        | InternalOperation::Strtoi
        | InternalOperation::Pmatch
        | InternalOperation::Charmatch
        | InternalOperation::Stop
        | InternalOperation::Inherits
        | InternalOperation::SignalCondition
        | InternalOperation::DefaultStop
        | InternalOperation::AddConditionHandlers
        | InternalOperation::ResetConditionHandlers
        | InternalOperation::GetErrorMessage => count == arity,
    };
    if !valid_arity {
        let suffix = if count == 1 { "" } else { "s" };
        return Err(arity_error(
            format!("{count} argument{suffix} passed to .Internal({name}) which requires {arity}"),
            call,
        ));
    }
    Ok(())
}

fn arity_error(message: String, call: Option<&crate::Sexp>) -> EvalError {
    match call {
        Some(call) => crate::SourceFault::arity(message, call),
        None => EvalError::Arity(message),
    }
}
