use crate::{CallArgument, EvalError, EvalFlow, RuntimeState, Sexp};

impl RuntimeState {
    pub(super) fn eval_extract_arguments(
        &mut self,
        target: &Sexp,
        arguments: &[CallArgument],
    ) -> Result<EvalFlow, EvalError> {
        if let Some(function) = self.lookup("[[")? {
            let mut supplied = vec![CallArgument::positional(target.clone())];
            supplied.extend_from_slice(arguments);
            let call = Sexp::Call {
                function: Box::new(Sexp::Symbol("[[".into())),
                arguments: supplied.clone(),
            };
            return self.call_value(function, &supplied, &call);
        }
        let EvalFlow::Value(value) = self.eval_flow(target)?;
        let args = self.eval_arguments(arguments, true, None)?;
        let mut indices = args
            .iter()
            .filter(|arg| arg.name.as_deref() != Some("exact"));
        let index = indices
            .next()
            .ok_or_else(|| EvalError::Domain("no index specified".into()))?;
        if indices.next().is_some() {
            return Err(EvalError::NotImplemented("multidimensional extraction"));
        }
        let exact: Vec<_> = args
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
        Ok(EvalFlow::Value(self.read_member(
            &value,
            crate::environment_members::MemberKey::Index(&index.value),
        )?))
    }

    pub(super) fn eval_replacement_call(
        &mut self,
        target: &Sexp,
        rhs: &Sexp,
    ) -> Result<EvalFlow, EvalError> {
        let EvalFlow::Value(replacement) = self.eval_flow(rhs)?;
        let (call, index) = match target {
            Sexp::Subset { value, index } => (value.as_ref(), index.as_deref()),
            other => (other, None),
        };
        let Sexp::Call {
            function,
            arguments,
        } = call
        else {
            return Err(EvalError::NotImplemented("this replacement target"));
        };
        let Sexp::Symbol(name) = function.as_ref() else {
            return Err(EvalError::NotImplemented("computed replacement function"));
        };
        if name == "attr" && index.is_none() {
            let [object, which] = arguments.as_slice() else {
                return Err(EvalError::Arity(
                    "attr replacement requires two arguments".into(),
                ));
            };
            let Sexp::Symbol(root) = &object.value else {
                return Err(EvalError::NotImplemented("nested attribute replacement"));
            };
            let current = self
                .lookup(root)?
                .ok_or_else(|| EvalError::UndefinedSymbol(root.clone()))?;
            let EvalFlow::Value(which) = self.eval_flow(&which.value)?;
            let name = crate::string_scalar(&which, "which")?;
            let updated = crate::list_attributes::set(&current, &name, &replacement)?;
            self.assign_local(root.clone(), updated)?;
            return Ok(EvalFlow::Value(replacement));
        }
        let [argument] = arguments.as_slice() else {
            return Err(EvalError::NotImplemented(
                "replacement calls with multiple arguments",
            ));
        };
        let Sexp::Symbol(root) = &argument.value else {
            return Err(EvalError::NotImplemented("nested replacement call target"));
        };
        let current = self
            .lookup(root)?
            .ok_or_else(|| EvalError::UndefinedSymbol(root.clone()))?;
        let updated = if let Some(index) = index {
            if name != "names" {
                return Err(EvalError::NotImplemented(
                    "subset replacement other than names",
                ));
            }
            let names = crate::names_value(&current)?;
            let EvalFlow::Value(index) = self.eval_flow(index)?;
            let names = crate::subassign_value(&names, Some(&index), &replacement)?;
            crate::names_assign_value(&current, &names)?
        } else {
            match name.as_str() {
                "attributes" => crate::list_attributes::replace(&current, &replacement)?,
                "class" => crate::list_attributes::replace_class(&current, &replacement)?,
                "names" => crate::names_assign_value(&current, &replacement)?,
                _ => return Err(EvalError::NotImplemented("this replacement function")),
            }
        };
        self.assign_local(root.clone(), updated)?;
        Ok(EvalFlow::Value(replacement))
    }
}
