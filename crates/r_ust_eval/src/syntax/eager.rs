use super::{arity, at_call, EvalError, RuntimeState, Sexp, SourceFault, SyntaxPrimitive};
use crate::{BinaryOp, EvaluatedArgument, UnaryOp};

impl RuntimeState {
    pub(crate) fn call_eager_syntax(
        &mut self,
        id: SyntaxPrimitive,
        arguments: &[EvaluatedArgument],
        call: &Sexp,
    ) -> Result<Sexp, EvalError> {
        use SyntaxPrimitive as P;
        let result = match id {
            P::Parenthesis => {
                let [value] = arguments else {
                    return Err(SourceFault::arity(arity(id, arguments.len(), 1), call));
                };
                Ok(value.value.clone())
            }
            P::Not => {
                let [value] = arguments else {
                    return Err(SourceFault::arity(arity(id, arguments.len(), 1), call));
                };
                crate::operators::eval_unary(UnaryOp::Not, &value.value)
            }
            P::And
            | P::Or
            | P::Equal
            | P::NotEqual
            | P::Less
            | P::LessEqual
            | P::Greater
            | P::GreaterEqual => {
                let [left, right] = arguments else {
                    return Err(SourceFault::arity(arity(id, arguments.len(), 2), call));
                };
                let op = match id {
                    P::And => BinaryOp::And,
                    P::Or => BinaryOp::Or,
                    P::Equal => BinaryOp::Equal,
                    P::NotEqual => BinaryOp::NotEqual,
                    P::Less => BinaryOp::Less,
                    P::LessEqual => BinaryOp::LessEqual,
                    P::Greater => BinaryOp::Greater,
                    P::GreaterEqual => BinaryOp::GreaterEqual,
                    _ => return Err(EvalError::Domain("binary syntax dispatch invariant".into())),
                };
                self.eval_binary(op, (&left.value, &right.value))
            }
            P::NamesAssign | P::ClassAssign | P::AttributesAssign => {
                let [object, value] = arguments else {
                    return Err(SourceFault::arity(arity(id, arguments.len(), 2), call));
                };
                match id {
                    P::NamesAssign => crate::names_assign_value(&object.value, &value.value),
                    P::ClassAssign => {
                        crate::list_attributes::replace_class(&object.value, &value.value)
                    }
                    P::AttributesAssign => {
                        crate::list_attributes::replace(&object.value, &value.value)
                    }
                    _ => {
                        return Err(EvalError::Domain(
                            "attribute syntax dispatch invariant".into(),
                        ))
                    }
                }
            }
            P::StorageModeAssign => {
                let [_, _] = arguments else {
                    return Err(SourceFault::arity(arity(id, arguments.len(), 2), call));
                };
                Err(EvalError::NotImplemented("storage mode replacement"))
            }
            P::AttributeAssign => {
                let [object, which, value] = arguments else {
                    return Err(SourceFault::arity(arity(id, arguments.len(), 3), call));
                };
                let name = crate::string_scalar(&which.value, "which")
                    .map_err(|error| at_call(error, call))?;
                crate::list_attributes::set(&object.value, &name, &value.value)
            }
            _ => {
                return Err(EvalError::NotImplemented(
                    "syntax special with evaluated-only apply arguments",
                ))
            }
        };
        self.visible = true;
        result.map_err(|error| at_call(error, call))
    }
}
