use crate::{CallArgument, EvalError, RuntimeState, Sexp};
use r_ust_sexp::InternalOperation;

pub fn resolve(name: &str) -> Option<InternalOperation> {
    match name {
        "environment" => Some(InternalOperation::Environment),
        "print.default" => Some(InternalOperation::PrintDefault),
        "new.env" => Some(InternalOperation::NewEnvironment),
        "parent.frame" => Some(InternalOperation::ParentFrame),
        "sys.frame" => Some(InternalOperation::SysFrame),
        "sys.function" => Some(InternalOperation::SysFunction),
        "sys.parent" => Some(InternalOperation::SysParent),
        "sys.nframe" => Some(InternalOperation::SysNFrame),
        "parent.env" => Some(InternalOperation::ParentEnvironment),
        "environmentName" => Some(InternalOperation::EnvironmentName),
        "formals" => Some(InternalOperation::Formals),
        "body" => Some(InternalOperation::Body),
        "bitwiseNot" => Some(InternalOperation::BitwiseNot),
        "bitwiseAnd" => Some(InternalOperation::BitwiseAnd),
        "bitwiseOr" => Some(InternalOperation::BitwiseOr),
        "bitwiseXor" => Some(InternalOperation::BitwiseXor),
        "bitwiseShiftL" => Some(InternalOperation::BitwiseShiftL),
        "bitwiseShiftR" => Some(InternalOperation::BitwiseShiftR),
        "encodeString" => Some(InternalOperation::EncodeString),
        "sequence" => Some(InternalOperation::Sequence),
        "utf8ToInt" => Some(InternalOperation::Utf8ToInt),
        "intToUtf8" => Some(InternalOperation::IntToUtf8),
        "findInterval" => Some(InternalOperation::FindInterval),
        "strtoi" => Some(InternalOperation::Strtoi),
        "pmatch" => Some(InternalOperation::Pmatch),
        "charmatch" => Some(InternalOperation::Charmatch),
        "as.vector" => Some(InternalOperation::AsVector),
        "vector" => Some(InternalOperation::Vector),
        "stop" => Some(InternalOperation::Stop),
        "inherits" => Some(InternalOperation::Inherits),
        "gettext" => Some(InternalOperation::Gettext),
        ".signalCondition" => Some(InternalOperation::SignalCondition),
        ".dfltStop" => Some(InternalOperation::DefaultStop),
        ".addCondHands" => Some(InternalOperation::AddConditionHandlers),
        ".resetCondHands" => Some(InternalOperation::ResetConditionHandlers),
        "geterrmessage" => Some(InternalOperation::GetErrorMessage),
        _ => None,
    }
}

impl RuntimeState {
    pub(crate) fn eval_internal_call(
        &mut self,
        arguments: &[CallArgument],
        outer_call: &Sexp,
    ) -> Result<Sexp, EvalError> {
        let [argument] = arguments else {
            return Err(crate::SourceFault::arity(
                format!(
                    "{} arguments passed to '.Internal' which requires 1",
                    arguments.len()
                ),
                outer_call,
            ));
        };
        if matches!(argument.value, Sexp::Parenthesized(_)) {
            return Err(crate::SourceFault::domain(
                "there is no .Internal function '('".into(),
                outer_call,
            ));
        }
        let Sexp::Call {
            function,
            arguments,
        } = &argument.value
        else {
            return Err(crate::SourceFault::domain(
                "invalid .Internal() argument".into(),
                outer_call,
            ));
        };
        let Sexp::Symbol(name) = function.as_ref() else {
            return Err(crate::SourceFault::domain(
                "invalid .Internal() argument".into(),
                outer_call,
            ));
        };
        let operation = resolve(name).ok_or_else(|| {
            crate::SourceFault::domain(
                format!("there is no .Internal function '{name}'"),
                outer_call,
            )
        })?;
        self.eval_internal(
            operation,
            arguments,
            super::InternalSite {
                outer: outer_call,
                inner: Some(&argument.value),
            },
        )
    }
}
