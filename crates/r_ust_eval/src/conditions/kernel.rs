use super::{head_value, ordinary_list, ListElement, PairListObject};
use crate::{EvalError, EvaluatedArgument, RuntimeState, Sexp};

impl RuntimeState {
    pub(crate) fn add_condition_handlers(
        &mut self,
        args: &[EvaluatedArgument],
    ) -> Result<Sexp, EvalError> {
        let [classes, handlers, parent, target, calling] = args else {
            return Err(EvalError::Arity(
                "5 arguments required by '.addCondHands'".into(),
            ));
        };
        let calling = crate::internal::as_logical(&calling.value).unwrap_or(false);
        let old = self.conditions.head.clone();
        if matches!(classes.value, Sexp::Nil) || matches!(handlers.value, Sexp::Nil) {
            self.visible = false;
            return Ok(head_value(&old));
        }
        let (Sexp::String(classes)
        | Sexp::AttributedCharacter {
            values: classes, ..
        }) = &classes.value
        else {
            return Err(crate::SourceFault::domain(
                "bad handler data".into(),
                &self.source_current_call(),
            ));
        };
        let handlers = match &handlers.value {
            Sexp::ListObject(object) => object.value()?,
            value => value.clone(),
        };
        let (Sexp::List(handlers)
        | Sexp::ClassedList {
            elements: handlers, ..
        }) = handlers
        else {
            return Err(crate::SourceFault::domain(
                "bad handler data".into(),
                &self.source_current_call(),
            ));
        };
        if classes.len() != handlers.len()
            || !matches!(parent.value, Sexp::Environment(_) | Sexp::Nil)
            || !matches!(target.value, Sexp::Environment(_) | Sexp::Nil)
        {
            return Err(crate::SourceFault::domain(
                "bad handler data".into(),
                &self.source_current_call(),
            ));
        }
        let token = match &self.conditions.token {
            Some(token) => token.clone(),
            None => ordinary_list(&self.global, vec![Sexp::Nil])?,
        };
        let result = ordinary_list(
            &self.global,
            vec![
                Sexp::Nil,
                Sexp::Nil,
                Sexp::Nil,
                Sexp::ListObject(token.clone()),
            ],
        )?;
        let mut next = old.clone();
        for (class, handler) in classes.iter().zip(handlers).rev() {
            let entry = ordinary_list(
                &self.global,
                vec![
                    Sexp::Char(class.clone()),
                    parent.value.clone(),
                    handler.value,
                    target.value.clone(),
                    Sexp::ListObject(result.clone()),
                ],
            )?;
            entry.set_gp_shared(u16::from(calling))?;
            next = Some(PairListObject::cons(
                &self.global,
                &ListElement {
                    name: None,
                    value: Sexp::ListObject(entry),
                },
                next.as_ref(),
            )?);
        }
        self.conditions.token = Some(token);
        self.conditions.head = next;
        self.visible = false;
        Ok(head_value(&old))
    }

    pub(crate) fn reset_condition_handlers(
        &mut self,
        args: &[EvaluatedArgument],
    ) -> Result<Sexp, EvalError> {
        let [value] = args else {
            return Err(EvalError::Arity(
                "1 argument required by '.resetCondHands'".into(),
            ));
        };
        self.conditions.reset(&self.global, &value.value)?;
        self.visible = false;
        Ok(Sexp::Nil)
    }
}
