mod access;
mod assignment;
mod control;
mod eager;
mod formals;
#[cfg(test)]
mod tests;

use crate::{CallArgument, EvalError, EvalFlow, RuntimeState, Sexp, SourceFault};
use r_ust_sexp::{SexpType, SyntaxPrimitive};

impl RuntimeState {
    pub(super) fn call_syntax(
        &mut self,
        id: SyntaxPrimitive,
        arguments: &[CallArgument],
        call: &Sexp,
    ) -> Result<EvalFlow, EvalError> {
        if id.sexptype() == SexpType::Builtin {
            let values = self.eval_arguments(arguments, true, Some(call))?;
            return self
                .call_eager_syntax(id, &values, call)
                .map(EvalFlow::Value);
        }
        use SyntaxPrimitive as P;
        match id {
            P::If
            | P::While
            | P::For
            | P::Repeat
            | P::Break
            | P::Next
            | P::Function
            | P::Block
            | P::AndScalar
            | P::OrScalar => self.call_control_syntax(id, arguments, call),
            P::Assign | P::EqualAssign | P::SuperAssign => {
                self.call_assignment_syntax(id, arguments, call)
            }
            P::Subset
            | P::Extract
            | P::Dollar
            | P::Subassign
            | P::ExtractAssign
            | P::DollarAssign => self.call_access_syntax(id, arguments, call),
            P::Parenthesis
            | P::NamesAssign
            | P::ClassAssign
            | P::AttributesAssign
            | P::AttributeAssign
            | P::StorageModeAssign
            | P::And
            | P::Or
            | P::Not
            | P::Equal
            | P::NotEqual
            | P::Less
            | P::LessEqual
            | P::Greater
            | P::GreaterEqual => Err(EvalError::Domain("eager syntax dispatch invariant".into())),
        }
    }

    pub(super) fn eval_syntax_operand(
        &mut self,
        expression: &Sexp,
        call: &Sexp,
    ) -> Result<EvalFlow, EvalError> {
        if matches!(expression, Sexp::MissingValue | Sexp::MissingArg(_)) {
            return Err(SourceFault::missing(
                String::new(),
                call.clone(),
                crate::conditions::MissingSite::EvaluatedSymbol,
            ));
        }
        self.eval_flow(expression)
    }
}

fn arity(id: SyntaxPrimitive, actual: usize, expected: usize) -> String {
    format!(
        "{actual} argument{} passed to '{}' which requires {expected}",
        if actual == 1 { "" } else { "s" },
        id.name()
    )
}

fn at_call(error: EvalError, call: &Sexp) -> EvalError {
    match error {
        EvalError::Type(message) => SourceFault::type_error(message, call),
        EvalError::Arity(message) => SourceFault::arity(message, call),
        EvalError::Domain(message) => SourceFault::domain(message, call),
        other => other,
    }
}
