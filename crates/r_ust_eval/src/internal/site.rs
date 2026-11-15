use crate::{EvalError, RuntimeState, Sexp};
use r_ust_sexp::InternalOperation;

#[derive(Clone, Copy)]
pub(crate) struct InternalSite<'a> {
    pub(crate) outer: &'a Sexp,
    pub(crate) inner: Option<&'a Sexp>,
}

impl RuntimeState {
    pub(super) fn internal_arity(
        &self,
        operation: InternalOperation,
        count: usize,
        inner: Option<&Sexp>,
    ) -> Result<(), EvalError> {
        let current = self.source_current_call();
        let call = match operation {
            InternalOperation::Gettext => Some(inner.unwrap_or(&Sexp::Nil)),
            InternalOperation::RunLengthEncode
            | InternalOperation::RunLengthDecode
            | InternalOperation::RowsumValidate
            | InternalOperation::RowsumAggregate
            | InternalOperation::EncodeStringWrapper => None,
            InternalOperation::Quit
            | InternalOperation::Identical
            | InternalOperation::Eval
            | InternalOperation::Assign
            | InternalOperation::Environment
            | InternalOperation::Sprintf
            | InternalOperation::Deparse
            | InternalOperation::Ngettext
            | InternalOperation::L10nInfo
            | InternalOperation::PrintDefault
            | InternalOperation::NewEnvironment
            | InternalOperation::ParentFrame
            | InternalOperation::SysFrame
            | InternalOperation::SysFunction
            | InternalOperation::SysParent
            | InternalOperation::SysNFrame
            | InternalOperation::ParentEnvironment
            | InternalOperation::EnvironmentName
            | InternalOperation::Formals
            | InternalOperation::Body
            | InternalOperation::BitwiseNot
            | InternalOperation::BitwiseAnd
            | InternalOperation::BitwiseOr
            | InternalOperation::BitwiseXor
            | InternalOperation::BitwiseShiftL
            | InternalOperation::BitwiseShiftR
            | InternalOperation::EncodeString
            | InternalOperation::Sequence
            | InternalOperation::Utf8ToInt
            | InternalOperation::IntToUtf8
            | InternalOperation::FindInterval
            | InternalOperation::Strtoi
            | InternalOperation::Pmatch
            | InternalOperation::Charmatch
            | InternalOperation::Vector
            | InternalOperation::IsVector
            | InternalOperation::AsVector
            | InternalOperation::Stop
            | InternalOperation::Inherits
            | InternalOperation::SignalCondition
            | InternalOperation::DefaultStop
            | InternalOperation::AddConditionHandlers
            | InternalOperation::ResetConditionHandlers
            | InternalOperation::GetErrorMessage
            | InternalOperation::Warning
            | InternalOperation::Lapply
            | InternalOperation::DoCall
            | InternalOperation::Get
            | InternalOperation::AddRestart
            | InternalOperation::GetRestart
            | InternalOperation::InvokeRestart
            | InternalOperation::DefaultWarning => Some(&current),
        };
        super::arity::check(operation, count, call)
    }
}
