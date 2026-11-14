use super::{member_values, EvalError, RuntimeState, Sexp};
use r_ust_sexp::GraphError;

#[derive(Clone, Copy)]
pub(super) enum MemberKey<'a> {
    Name(&'a str),
    Index(&'a Sexp),
}

enum Access {
    Read,
    Write,
}

impl RuntimeState {
    pub(super) fn read_member(
        &mut self,
        object: &Sexp,
        key: MemberKey<'_>,
    ) -> Result<Sexp, EvalError> {
        match object {
            Sexp::Environment(frame) => {
                let name = environment_name(key, Access::Read)?;
                match frame.get_local(name)? {
                    Some(Sexp::MissingArg(_)) => Ok(Sexp::MissingValue),
                    Some(value) => self.force_value(value),
                    None => Ok(Sexp::Nil),
                }
            }
            other => match key {
                MemberKey::Name(name) => member_values::dollar_value(other, name),
                MemberKey::Index(index) => member_values::extract_value(other, index),
            },
        }
    }
}

pub(super) fn write_member(
    object: &Sexp,
    key: MemberKey<'_>,
    value: &Sexp,
) -> Result<Sexp, EvalError> {
    match object {
        Sexp::Environment(frame) => {
            let name = environment_name(key, Access::Write)?;
            frame
                .insert(name.into(), value)
                .map_err(|error| member_graph_error(error, name))?;
            Ok(Sexp::Environment(frame.clone()))
        }
        other => match key {
            MemberKey::Name(name) => member_values::dollar_assign_value(other, name, value),
            MemberKey::Index(index) => member_values::extract_assign_value(other, index, value),
        },
    }
}

fn environment_name(key: MemberKey<'_>, access: Access) -> Result<&str, EvalError> {
    let name = match key {
        MemberKey::Name(name) => name,
        MemberKey::Index(Sexp::String(values) | Sexp::AttributedCharacter { values, .. })
            if values.len() == 1 =>
        {
            values[0].as_deref().unwrap_or("NA")
        }
        MemberKey::Index(_) => {
            return Err(EvalError::Type(
                match access {
                    Access::Read => "wrong arguments for subsetting an environment",
                    Access::Write => "wrong args for environment subassignment",
                }
                .into(),
            ))
        }
    };
    if name.is_empty() {
        return Err(EvalError::Domain(
            "attempt to use zero-length variable name".into(),
        ));
    }
    Ok(name)
}

pub(super) fn member_graph_error(error: GraphError, name: &str) -> EvalError {
    match error {
        GraphError::BindingLocked => EvalError::Domain(format!(
            "cannot change value of locked binding for '{name}'"
        )),
        GraphError::EnvironmentLocked => EvalError::Domain(format!(
            "cannot add binding of '{name}' to the base environment"
        )),
        GraphError::EmptyEnvironment => {
            EvalError::Domain("cannot assign values in the empty environment".into())
        }
        GraphError::Unrooted
        | GraphError::ForeignHeap
        | GraphError::StaleNode
        | GraphError::WrongKind
        | GraphError::IndexOutOfBounds
        | GraphError::InvalidCharacter
        | GraphError::PromiseUnderEvaluation
        | GraphError::PromiseNotEvaluating => EvalError::Graph(error),
    }
}
