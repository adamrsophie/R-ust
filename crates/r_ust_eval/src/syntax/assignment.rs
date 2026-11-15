use super::{CallArgument, EvalError, EvalFlow, RuntimeState, Sexp, SourceFault, SyntaxPrimitive};
use crate::environment::has_builtin;
use r_ust_sexp::{Force, PromiseState};

struct Step {
    name: String,
    arguments: Vec<CallArgument>,
    source: Sexp,
}

struct Place {
    root: String,
    value: Sexp,
    steps: Vec<Step>,
}

impl RuntimeState {
    pub(super) fn call_assignment_syntax(
        &mut self,
        id: SyntaxPrimitive,
        arguments: &[CallArgument],
        call: &Sexp,
    ) -> Result<EvalFlow, EvalError> {
        let [target, rhs] = arguments else {
            return Err(SourceFault::arity(
                format!("incorrect number of arguments to \"{}\"", id.name()),
                call,
            ));
        };
        if let Some(name) = assignment_name(&target.value) {
            if name.is_empty() {
                return Err(SourceFault::domain(
                    "attempt to use zero-length variable name".into(),
                    call,
                ));
            }
            let EvalFlow::Value(value) = self.eval_syntax_operand(&rhs.value, call)?;
            if id == SyntaxPrimitive::SuperAssign {
                self.assign_super(name, value.clone())?;
            } else {
                self.assign_local(name, value.clone())?;
            }
            self.visible = false;
            return Ok(EvalFlow::Value(value));
        }
        let target = crate::syntax_raw::call(&target.value).unwrap_or_else(|| target.value.clone());
        if !matches!(target, Sexp::Call { .. }) {
            return Err(SourceFault::type_error(
                "invalid (do_set) left-hand side to assignment".into(),
                call,
            ));
        }
        if id == SyntaxPrimitive::SuperAssign {
            return Err(EvalError::NotImplemented("complex superassignment"));
        }
        let EvalFlow::Value(replacement) = self.eval_syntax_operand(&rhs.value, call)?;
        let place = self.prepare_syntax_place(&target, false)?;
        let mut updated = replacement.clone();
        for step in place.steps.into_iter().rev() {
            let setter = format!("{}<-", step.name);
            let source = replacement_source(&step.source, &rhs.value);
            let selected = self.require_source_function(&setter, has_builtin(&setter), &source)?;
            let mut supplied = step.arguments;
            supplied.push(CallArgument::named(
                "value",
                self.cached_syntax_value(&rhs.value, &updated)?,
            ));
            let EvalFlow::Value(value) = self.call_resolved(selected, &supplied, &source)?;
            updated = value;
        }
        self.assign_local(place.root, updated)?;
        self.visible = false;
        Ok(EvalFlow::Value(replacement))
    }

    fn prepare_syntax_place(&mut self, target: &Sexp, read: bool) -> Result<Place, EvalError> {
        if let Some(root) = assignment_name(target) {
            let local = self.current.contains_local(&root)?;
            let value = self
                .lookup(&root)?
                .ok_or_else(|| SourceFault::undefined_symbol(&root, &self.source_lexical_call()))?;
            if !local {
                self.assign_local(root.clone(), value.clone())?;
            }
            return Ok(Place {
                root,
                value,
                steps: Vec::new(),
            });
        }
        let target = crate::syntax_raw::call(target).unwrap_or_else(|| target.clone());
        let Sexp::Call {
            function,
            arguments,
        } = &target
        else {
            return Err(SourceFault::type_error(
                "invalid (do_set) left-hand side to assignment".into(),
                &target,
            ));
        };
        let Sexp::Symbol(name) = function.as_ref() else {
            return Err(EvalError::NotImplemented("computed replacement function"));
        };
        let Some((object, selectors)) = arguments.split_first() else {
            return Err(SourceFault::arity(
                "invalid function in complex assignment".into(),
                &target,
            ));
        };
        let mut place = self.prepare_syntax_place(&object.value, true)?;
        let mut supplied = vec![CallArgument {
            name: object.name.clone(),
            value: self.cached_syntax_value(&object.value, &place.value)?,
        }];
        for selector in selectors {
            supplied.push(CallArgument {
                name: selector.name.clone(),
                value: selector.value.clone(),
            });
        }
        if read {
            let selected = self.require_source_function(name, has_builtin(name), &target)?;
            let EvalFlow::Value(value) = self.call_resolved(selected, &supplied, &target)?;
            place.value = value;
        }
        place.steps.push(Step {
            name: name.clone(),
            arguments: supplied,
            source: target,
        });
        Ok(place)
    }

    fn cached_syntax_value(&self, expression: &Sexp, value: &Sexp) -> Result<Sexp, EvalError> {
        let state =
            PromiseState::new(expression, &self.current, self.control.origin.clone(), true)?;
        match state.begin_force()? {
            Force::Evaluate(lease) => lease.complete(value)?,
            Force::Cached(_) => {
                return Err(EvalError::Domain(
                    "new replacement promise already cached".into(),
                ))
            }
        }
        Ok(Sexp::Promise { state })
    }
}

fn assignment_name(value: &Sexp) -> Option<String> {
    match value {
        Sexp::Symbol(name) => Some(name.clone()),
        Sexp::String(values) | Sexp::AttributedCharacter { values, .. } => values
            .first()
            .map(|name| name.clone().unwrap_or_else(|| "NA".into())),
        _ => None,
    }
}

fn replacement_source(target: &Sexp, rhs: &Sexp) -> Sexp {
    match target {
        Sexp::Call {
            function,
            arguments,
        } => {
            let mut arguments = arguments.clone();
            arguments.push(CallArgument::named("value", rhs.clone()));
            let function = match function.as_ref() {
                Sexp::Symbol(name) => Sexp::Symbol(format!("{name}<-")),
                other => other.clone(),
            };
            Sexp::Call {
                function: Box::new(function),
                arguments,
            }
        }
        other => other.clone(),
    }
}
