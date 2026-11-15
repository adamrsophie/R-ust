use crate::{
    environment::ApplyFunction, CallArgument, EvalError, EvalFlow, ListElement, RuntimeState, Sexp,
    SourceFault,
};

impl RuntimeState {
    pub(super) fn internal_lapply(
        &mut self,
        arguments: &[CallArgument],
        call: &Sexp,
    ) -> Result<Sexp, EvalError> {
        let Sexp::Symbol(x) = &arguments[0].value else {
            return Err(SourceFault::domain(
                "argument must be a symbol".into(),
                call,
            ));
        };
        let EvalFlow::Value(input) = self.eval_flow(&arguments[0].value)?;
        let Sexp::Symbol(function) = &arguments[1].value else {
            return Err(SourceFault::domain(
                "argument must be a symbol".into(),
                call,
            ));
        };
        let length = crate::value_len(&input)?;
        let integer_indices = i32::try_from(length).is_ok();
        let mut real_index = 1.0;
        let names = crate::names_value(&input)?;
        let names_present = !matches!(names, Sexp::Nil);
        let names = names.character_values().unwrap_or(&[]);
        let extraction = Sexp::Call {
            function: Box::new(Sexp::Symbol("[[".into())),
            arguments: vec![
                CallArgument::positional(Sexp::Symbol(x.clone())),
                CallArgument::positional(Sexp::Symbol("i".into())),
            ],
        };
        let callback = Sexp::Call {
            function: Box::new(Sexp::Symbol(function.clone())),
            arguments: vec![
                CallArgument::positional(extraction),
                CallArgument::positional(Sexp::Symbol("...".into())),
            ],
        };
        self.assign_local("i".into(), Sexp::Integer(vec![Some(1)]))?;
        let mut output = Vec::with_capacity(length);
        for index in 0..length {
            let value = if integer_indices {
                Sexp::Integer(vec![Some(i32::try_from(index + 1).map_err(|_| {
                    EvalError::Control("lapply integer index overflow".into())
                })?)])
            } else {
                Sexp::Real(vec![real_index])
            };
            real_index += 1.0;
            self.assign_local("i".into(), value)?;
            let value = self.force_first_and_call(&callback)?;
            output.push(ListElement {
                name: names.get(index).cloned().flatten(),
                value,
            });
        }
        Ok(Sexp::list_with_metadata(output, names_present, None))
    }

    fn force_first_and_call(&mut self, call: &Sexp) -> Result<Sexp, EvalError> {
        let Sexp::Call {
            function,
            arguments,
        } = call
        else {
            return Err(EvalError::Control(
                "invalid force-and-call expression".into(),
            ));
        };
        let Sexp::Symbol(name) = function.as_ref() else {
            return Err(EvalError::Control("invalid force-and-call function".into()));
        };
        let selected = self.require_source_function(name, false, call)?;
        let flow = match selected {
            ApplyFunction::Closure(callable) => {
                let invocation = self.prepare_closure_call(callable, call.clone(), arguments)?;
                if let Some(first) = invocation.arguments.first() {
                    self.force_value(first.value.clone())?;
                }
                self.invoke_closure(invocation)?
            }
            selected => self.call_resolved(selected, arguments, call)?,
        };
        let EvalFlow::Value(value) = flow;
        Ok(value)
    }
}
