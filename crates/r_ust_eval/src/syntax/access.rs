use super::{
    arity, at_call, CallArgument, EvalError, EvalFlow, RuntimeState, Sexp, SourceFault,
    SyntaxPrimitive,
};
use crate::environment_members::{write_member, MemberKey};
use crate::{expand_dots_call_arguments, EvaluatedArgument};

impl RuntimeState {
    pub(super) fn call_access_syntax(
        &mut self,
        id: SyntaxPrimitive,
        arguments: &[CallArgument],
        call: &Sexp,
    ) -> Result<EvalFlow, EvalError> {
        use SyntaxPrimitive as P;
        if matches!(id, P::Dollar | P::DollarAssign) {
            return self.syntax_dollar(id, arguments, call);
        }
        let arguments = self.syntax_access_arguments(arguments, call)?;
        let result = match id {
            P::Subset => subset(&arguments),
            P::Extract => self.syntax_extract(&arguments),
            P::Subassign => subassign(&arguments),
            P::ExtractAssign => {
                let [object, index, value] = arguments.as_slice() else {
                    return Err(SourceFault::arity(arity(id, arguments.len(), 3), call));
                };
                write_member(&object.value, MemberKey::Index(&index.value), &value.value)
            }
            _ => Err(EvalError::Domain("access syntax dispatch invariant".into())),
        };
        self.visible = true;
        result
            .map(EvalFlow::Value)
            .map_err(|error| at_call(error, call))
    }

    fn syntax_access_arguments(
        &mut self,
        arguments: &[CallArgument],
        call: &Sexp,
    ) -> Result<Vec<EvaluatedArgument>, EvalError> {
        let arguments = expand_dots_call_arguments(arguments, &self.current)?;
        arguments
            .iter()
            .enumerate()
            .map(|(index, argument)| {
                let value = if matches!(argument.value, Sexp::MissingValue | Sexp::MissingArg(_)) {
                    if index == 0 {
                        return Err(SourceFault::arity("argument 1 is empty".into(), call));
                    }
                    Sexp::MissingValue
                } else {
                    let EvalFlow::Value(value) = self.eval_flow(&argument.value)?;
                    value
                };
                Ok(EvaluatedArgument {
                    name: argument.name.clone(),
                    value,
                })
            })
            .collect()
    }

    fn syntax_extract(&mut self, arguments: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
        let Some((object, arguments)) = arguments.split_first() else {
            return Err(EvalError::Domain("no index specified".into()));
        };
        let indices: Vec<_> = arguments
            .iter()
            .filter(|arg| arg.name.as_deref() != Some("exact"))
            .collect();
        let [index] = indices.as_slice() else {
            return Err(if indices.is_empty() {
                EvalError::Domain("no index specified".into())
            } else {
                EvalError::NotImplemented("multidimensional extraction")
            });
        };
        let exact: Vec<_> = arguments
            .iter()
            .filter(|arg| arg.name.as_deref() == Some("exact"))
            .collect();
        if exact.len() > 1 {
            return Err(EvalError::Domain("incorrect number of subscripts".into()));
        }
        if let Some(exact) = exact.first() {
            if crate::logical_scalar(&exact.value, "exact")? != Some(true) {
                return Err(EvalError::NotImplemented("partial list extraction"));
            }
        }
        self.read_member(&object.value, MemberKey::Index(&index.value))
    }

    fn syntax_dollar(
        &mut self,
        id: SyntaxPrimitive,
        arguments: &[CallArgument],
        call: &Sexp,
    ) -> Result<EvalFlow, EvalError> {
        let expected = if id == SyntaxPrimitive::Dollar { 2 } else { 3 };
        if arguments.len() != expected {
            return Err(SourceFault::arity(
                arity(id, arguments.len(), expected),
                call,
            ));
        }
        let name = member_name(&arguments[1].value).map_err(|error| at_call(error, call))?;
        let EvalFlow::Value(object) = self.eval_syntax_operand(&arguments[0].value, call)?;
        let result = if let Some(value) = arguments.get(2) {
            let EvalFlow::Value(value) = self.eval_syntax_operand(&value.value, call)?;
            write_member(&object, MemberKey::Name(&name), &value)
        } else {
            self.read_member(&object, MemberKey::Name(&name))
        };
        self.visible = true;
        result
            .map(EvalFlow::Value)
            .map_err(|error| at_call(error, call))
    }
}

fn member_name(value: &Sexp) -> Result<String, EvalError> {
    match value {
        Sexp::Symbol(name) => Ok(name.clone()),
        Sexp::String(values) | Sexp::AttributedCharacter { values, .. } if values.len() == 1 => {
            Ok(values[0].clone().unwrap_or_else(|| "NA".into()))
        }
        _ => Err(EvalError::Type("invalid subscript type".into())),
    }
}

fn index(value: &Sexp) -> Option<&Sexp> {
    (!matches!(value, Sexp::MissingValue | Sexp::MissingArg(_))).then_some(value)
}

fn subset(arguments: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let Some((object, arguments)) = arguments.split_first() else {
        return Err(EvalError::Arity("argument 1 is empty".into()));
    };
    let drops: Vec<_> = arguments
        .iter()
        .filter(|argument| argument.name.as_deref() == Some("drop"))
        .collect();
    if drops.len() > 1 {
        return Err(EvalError::Domain("incorrect number of subscripts".into()));
    }
    if let Some(drop) = drops.first() {
        if matches!(object.value, Sexp::Matrix { .. })
            && crate::logical_scalar(&drop.value, "drop")? == Some(false)
        {
            return Err(EvalError::NotImplemented(
                "dimension-preserving matrix subset",
            ));
        }
    }
    let indices: Vec<_> = arguments
        .iter()
        .filter(|argument| argument.name.as_deref() != Some("drop"))
        .collect();
    match indices.as_slice() {
        [] => crate::subset_value(&object.value, None),
        [one] => crate::subset_value(&object.value, index(&one.value)),
        [row, col] => crate::subset_2d_value(&object.value, index(&row.value), index(&col.value)),
        _ => Err(EvalError::NotImplemented("subset dimensions beyond two")),
    }
}

fn subassign(arguments: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    match arguments {
        [object, one, value] => {
            crate::subassign_value(&object.value, index(&one.value), &value.value)
        }
        [object, row, col, value] => crate::matrix_subassign_value(
            &object.value,
            index(&row.value),
            index(&col.value),
            &value.value,
        ),
        _ => Err(EvalError::Arity("incorrect number of subscripts".into())),
    }
}
