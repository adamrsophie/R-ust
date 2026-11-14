mod calling_error;
mod calls;
mod entry;
mod fault_calls;
mod faults;
mod source_fault;
mod symbol_faults;
pub(crate) use source_fault::MissingSite;
pub use source_fault::SourceFault;
mod kernel;
mod matching;
mod on_exit;
mod signaling;
mod stack;
mod target;
mod unwind;

#[cfg(test)]
mod tests;

use r_ust_sexp::{Environment, EvaluationEnvironmentId, ListElement, ListObject, PairListObject};

use crate::{EvalError, Sexp};

#[derive(Debug, Default)]
pub(crate) struct Conditions {
    pub(crate) head: Option<PairListObject>,
    pub(crate) token: Option<ListObject>,
    pub(crate) message: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ConditionJump {
    pub(crate) target: EvaluationEnvironmentId,
    pub(crate) result: ListObject,
    pub(crate) token: Option<ListObject>,
    pub(crate) message: Option<String>,
}

fn ordinary_list(owner: &Environment, values: Vec<Sexp>) -> Result<ListObject, EvalError> {
    let value = Sexp::List(
        values
            .into_iter()
            .map(|value| ListElement { name: None, value })
            .collect(),
    );
    Ok(ListObject::from_value(owner, &value)?)
}

fn bad_stack() -> EvalError {
    EvalError::Domain("bad handler data".into())
}

fn head_value(head: &Option<PairListObject>) -> Sexp {
    head.clone().map_or(Sexp::Nil, Sexp::PairListObject)
}
