use crate::{
    ControlTransfer, EvalError, EvaluatedArgument, ListElement, RuntimeState, Sexp, SourceFault,
};
use r_ust_sexp::{ListObject, PairListObject};

impl RuntimeState {
    pub(crate) fn add_restart(
        &mut self,
        arguments: &[EvaluatedArgument],
    ) -> Result<Sexp, EvalError> {
        let restart = restart_value(&arguments[0].value, &self.source_current_call())?;
        let object = ListObject::from_value(&self.global, &restart)?;
        let head = PairListObject::cons(
            &self.global,
            &ListElement {
                name: None,
                value: Sexp::ListObject(object),
            },
            self.conditions.restarts.as_ref(),
        )?;
        self.conditions.restarts = Some(head);
        Ok(Sexp::Nil)
    }

    pub(crate) fn get_restart(
        &mut self,
        arguments: &[EvaluatedArgument],
    ) -> Result<Sexp, EvalError> {
        let mut index = self.scalar_integer_with_warning(&arguments[0].value)?;
        let mut cursor = self.conditions.restarts.clone();
        while index.is_some_and(|index| index > 1) {
            let Some(node) = cursor.clone() else { break };
            cursor = node.cdr()?;
            index = index.map(|index| index - 1);
        }
        if let Some(node) = cursor {
            return Ok(node.car()?.value);
        }
        Ok(if index == Some(1) {
            Sexp::ClassedList {
                elements: vec![
                    ListElement {
                        name: None,
                        value: Sexp::String(vec![Some("abort".into())]),
                    },
                    ListElement {
                        name: None,
                        value: Sexp::Nil,
                    },
                ],
                class: Box::new(Sexp::String(vec![Some("restart".into())])),
                class_first: true,
            }
        } else {
            Sexp::Nil
        })
    }

    pub(crate) fn invoke_restart(
        &mut self,
        arguments: &[EvaluatedArgument],
    ) -> Result<Sexp, EvalError> {
        let call = self.source_current_call();
        let restart = restart_value(&arguments[0].value, &call)?;
        self.global.validate_import(&restart)?;
        self.global.validate_import(&arguments[1].value)?;
        let exit = restart_exit(&restart)?;
        if matches!(exit, Sexp::Nil) {
            return Err(EvalError::NotImplemented(
                "top-level abort restart transfer",
            ));
        }
        while let Some(node) = self.conditions.restarts.clone() {
            let candidate = restart_value(&node.car()?.value, &call)?;
            self.conditions.restarts = node.cdr()?;
            if let (Sexp::Environment(expected), Sexp::Environment(found)) =
                (&exit, restart_exit(&candidate)?)
            {
                if *expected == found {
                    let target = self.control.condition_target(expected)?;
                    return Err(EvalError::Transfer(ControlTransfer::Return(
                        target,
                        arguments[1].value.clone(),
                    )));
                }
            }
        }
        Err(SourceFault::domain("restart not on stack".into(), &call))
    }
}

fn restart_value(value: &Sexp, call: &Sexp) -> Result<Sexp, EvalError> {
    let projection = crate::ordinary_values::read(value)?;
    if projection
        .as_ref()
        .ordinary_list()
        .is_some_and(|view| view.elements.len() >= 2)
    {
        Ok(projection.as_ref().clone())
    } else {
        Err(SourceFault::domain("bad restart".into(), call))
    }
}

fn restart_exit(value: &Sexp) -> Result<Sexp, EvalError> {
    value
        .ordinary_list()
        .and_then(|view| view.elements.get(1))
        .map(|element| element.value.clone())
        .ok_or_else(|| EvalError::Control("invalid restart representation".into()))
}
