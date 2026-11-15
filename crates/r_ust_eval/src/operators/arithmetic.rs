use super::{eval_unary, unary_numbers};
use crate::{BinaryOp, EvalError, EvaluatedArgument, RuntimeState, Sexp, UnaryOp};
use r_ust_sexp::PrimitiveId;

impl RuntimeState {
    pub(crate) fn call_arithmetic(
        &mut self,
        id: PrimitiveId,
        arguments: &[EvaluatedArgument],
    ) -> Result<Sexp, EvalError> {
        let op = match id {
            PrimitiveId::Add => BinaryOp::Add,
            PrimitiveId::Subtract => BinaryOp::Subtract,
            PrimitiveId::Multiply => BinaryOp::Multiply,
            PrimitiveId::Divide => BinaryOp::Divide,
            PrimitiveId::Power => BinaryOp::Power,
            PrimitiveId::Modulo => BinaryOp::Modulo,
            PrimitiveId::IntegerDivide => BinaryOp::IntegerDivide,
            PrimitiveId::Colon => BinaryOp::Sequence,
            PrimitiveId::Syntax(_)
            | PrimitiveId::InternalEval
            | PrimitiveId::Internal
            | PrimitiveId::Call
            | PrimitiveId::IsPairList
            | PrimitiveId::Sum
            | PrimitiveId::List
            | PrimitiveId::AsCharacter
            | PrimitiveId::GlobalEnv
            | PrimitiveId::BaseEnv
            | PrimitiveId::EmptyEnv
            | PrimitiveId::IsEnvironment
            | PrimitiveId::Primitive
            | PrimitiveId::Length
            | PrimitiveId::Invisible
            | PrimitiveId::Return
            | PrimitiveId::Missing
            | PrimitiveId::Quote
            | PrimitiveId::Expression
            | PrimitiveId::IsExpression
            | PrimitiveId::Substitute
            | PrimitiveId::NamespaceExport
            | PrimitiveId::NamespaceInternal
            | PrimitiveId::AsCall
            | PrimitiveId::AsEnvironment
            | PrimitiveId::IsSymbol
            | PrimitiveId::UseMethod
            | PrimitiveId::OnExit
            | PrimitiveId::Nargs => {
                return Err(EvalError::NotImplemented("non-arithmetic primitive"))
            }
        };
        if id == PrimitiveId::Colon && arguments.len() != 2 {
            return Err(EvalError::Arity(format!(
                "{} argument{} passed to ':' which requires 2",
                arguments.len(),
                if arguments.len() == 1 { "" } else { "s" }
            )));
        }
        match arguments {
            [left, right] => self.eval_binary(op, (&left.value, &right.value)),
            [value] => match id {
                PrimitiveId::Add => eval_unary(UnaryOp::Plus, &value.value),
                PrimitiveId::Subtract => eval_unary(UnaryOp::Minus, &value.value),
                PrimitiveId::Multiply
                | PrimitiveId::Divide
                | PrimitiveId::Power
                | PrimitiveId::Modulo
                | PrimitiveId::IntegerDivide => {
                    unary_numbers(&value.value)?;
                    Err(EvalError::Type("invalid unary operator".into()))
                }
                PrimitiveId::Syntax(_)
                | PrimitiveId::InternalEval
                | PrimitiveId::Internal
                | PrimitiveId::Call
                | PrimitiveId::IsPairList
                | PrimitiveId::Colon
                | PrimitiveId::GlobalEnv
                | PrimitiveId::BaseEnv
                | PrimitiveId::EmptyEnv
                | PrimitiveId::IsEnvironment
                | PrimitiveId::Sum
                | PrimitiveId::List
                | PrimitiveId::AsCharacter
                | PrimitiveId::Primitive
                | PrimitiveId::Length
                | PrimitiveId::Invisible
                | PrimitiveId::Return
                | PrimitiveId::Missing
                | PrimitiveId::Quote
                | PrimitiveId::Expression
                | PrimitiveId::IsExpression
                | PrimitiveId::Substitute
                | PrimitiveId::NamespaceExport
                | PrimitiveId::NamespaceInternal
                | PrimitiveId::AsCall
                | PrimitiveId::AsEnvironment
                | PrimitiveId::IsSymbol
                | PrimitiveId::UseMethod
                | PrimitiveId::OnExit
                | PrimitiveId::Nargs => Err(EvalError::NotImplemented("non-unary primitive")),
            },
            _ => Err(EvalError::Arity(
                "operator needs one or two arguments".into(),
            )),
        }
    }
}
