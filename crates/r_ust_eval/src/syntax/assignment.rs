mod ownership;

use super::{CallArgument, EvalError, EvalFlow, RuntimeState, Sexp, SourceFault, SyntaxPrimitive};
use crate::environment::has_builtin;
use r_ust_sexp::{Environment, Force, ListObject, PrimitiveId, PromiseState};

struct Step {
    name: String,
    arguments: Vec<CallArgument>,
    source: Sexp,
    reusable: Option<ListObject>,
}

struct Place {
    root: String,
    frame: Environment,
    unshared: bool,
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
        let EvalFlow::Value(replacement) = self.eval_syntax_operand(&rhs.value, call)?;
        let _rhs_lease = self.current.retain_value(&replacement)?;
        let mut place =
            self.prepare_syntax_place(&target, (false, id == SyntaxPrimitive::SuperAssign))?;
        let mut updated = replacement.clone();
        while !place.steps.is_empty() {
            let argument =
                CallArgument::named("value", self.cached_syntax_value(&rhs.value, &updated)?);
            let step = place.steps.last_mut().unwrap();
            step.arguments.push(argument);
            let setter = format!("{}<-", step.name);
            let source = replacement_source(&step.source, &rhs.value);
            let ownership = place.ownership()?;
            let selected = self.require_source_function(&setter, has_builtin(&setter), &source)?;
            let builtin_setter = matches!(
                &selected,
                crate::environment::ApplyFunction::Primitive(PrimitiveId::Syntax(
                    SyntaxPrimitive::DollarAssign
                        | SyntaxPrimitive::ExtractAssign
                        | SyntaxPrimitive::Subassign
                ))
            );
            let supplied = &place.steps.last().unwrap().arguments;
            let EvalFlow::Value(value) = self.call_resolved(selected, supplied, &source)?;
            place.check_ownership(ownership)?;
            let step = place.steps.pop().unwrap();
            updated = if builtin_setter {
                if let Some(object) = step.reusable {
                    let projected = crate::ordinary_values::read(&value)?;
                    if projected.ordinary_list().is_some() {
                        object.replace_value_shared(projected.as_ref())?;
                        Sexp::ListObject(object)
                    } else {
                        value
                    }
                } else {
                    value
                }
            } else {
                value
            };
        }
        place
            .frame
            .insert(place.root.clone(), &updated)
            .map_err(|error| crate::environment_members::member_graph_error(error, &place.root))?;
        self.visible = false;
        Ok(EvalFlow::Value(replacement))
    }

    fn prepare_syntax_place(
        &mut self,
        target: &Sexp,
        (read, superassign): (bool, bool),
    ) -> Result<Place, EvalError> {
        if let Some(root) = assignment_name(target) {
            let frame = if superassign {
                self.replacement_parent(&root)?
            } else {
                self.current.clone()
            };
            let local = frame.contains_local(&root)?;
            let value = frame
                .get(&root)?
                .ok_or_else(|| SourceFault::undefined_symbol(&root, &self.source_lexical_call()))?;
            let value = self.force_value(value)?;
            if !local && !superassign {
                self.assign_local(root.clone(), value.clone())?;
            }
            let unshared = unshared_list(&value)?;
            return Ok(Place {
                root,
                frame,
                unshared,
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
            return Err(SourceFault::domain(
                "invalid (NULL) left side of assignment".into(),
                &target,
            ));
        };
        let mut place = self.prepare_syntax_place(&object.value, (true, superassign))?;
        let reusable = if place.unshared {
            match &place.value {
                Sexp::ListObject(object) => Some(object.clone()),
                _ => None,
            }
        } else {
            None
        };
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
        place.steps.push(Step {
            name: name.clone(),
            arguments: supplied,
            source: target.clone(),
            reusable,
        });
        if read {
            let ownership = place.ownership()?;
            let selected = self.require_source_function(name, has_builtin(name), &target)?;
            let supplied = &place.steps.last().unwrap().arguments;
            let EvalFlow::Value(value) = self.call_resolved(selected, supplied, &target)?;
            place.check_ownership(ownership)?;
            place.unshared = place.unshared && unshared_list(&value)?;
            place.value = value;
        }
        Ok(place)
    }

    fn replacement_parent(&self, name: &str) -> Result<Environment, EvalError> {
        let mut parent = self.current.parent()?;
        while let Some(frame) = parent {
            if frame.contains_local(name)? {
                return Ok(frame);
            }
            parent = frame.parent()?;
        }
        Ok(self.global.clone())
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

fn unshared_list(value: &Sexp) -> Result<bool, EvalError> {
    Ok(match value {
        Sexp::ListObject(object) => {
            object.is_runtime_value()? && object.semantic_reference_count()? <= 1
        }
        _ => false,
    })
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
