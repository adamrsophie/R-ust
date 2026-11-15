use crate::{CallArgument, EvalError, ListElement, RuntimeState, Sexp};
use r_ust_sexp::EnvironmentRole;

impl RuntimeState {
    pub(crate) fn signal_partial_warning(
        &mut self,
        argument: &str,
        formal: &str,
        call: &Sexp,
    ) -> Result<(), EvalError> {
        let call = r_ust_sexp::canonical_actual_call(call)
            .map_err(|_| EvalError::NotImplemented("warning call source representation"))?;
        let condition = Sexp::list_with_metadata(
            vec![
                field(
                    "message",
                    Sexp::String(vec![Some(format!(
                        "partial argument match of '{argument}' to '{formal}'"
                    ))]),
                ),
                field("call", call),
                field("argument", Sexp::Symbol(argument.into())),
                field("formal", Sexp::Symbol(formal.into())),
            ],
            true,
            Some((
                Sexp::String(vec![
                    Some("partialMatchWarning".into()),
                    Some("warning".into()),
                    Some("condition".into()),
                ]),
                false,
            )),
        );
        crate::base::initialize(&self.current)?;
        let environment = self
            .global
            .session_root(EnvironmentRole::BaseNamespace)?
            .child()?;
        environment.insert("cond".into(), &condition)?;
        let expression = Sexp::Call {
            function: Box::new(Sexp::Symbol("warning".into())),
            arguments: vec![CallArgument::positional(Sexp::Symbol("cond".into()))],
        };
        let visible = self.visible;
        self.eval_call_in_environment(&expression, &environment)?;
        self.visible = visible;
        Ok(())
    }
}

fn field(name: &str, value: Sexp) -> ListElement {
    ListElement {
        name: Some(name.into()),
        value,
    }
}
