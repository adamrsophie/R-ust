use r_ust_sexp::PrimitiveId;
mod accessor;
mod construction;
mod control;
mod display;
mod environment;
mod invocation;
mod namespace;
mod unclass;
pub(super) use display::format_primitive;
pub(crate) mod names;

use super::{
    ApplyFunction, CallArgument, EvalError, EvalFlow, EvaluatedArgument, RuntimeState, Sexp,
};

impl RuntimeState {
    pub(super) fn eval_apply_extra(
        &mut self,
        function: &ApplyFunction,
        arguments: &[CallArgument],
    ) -> Result<Vec<EvaluatedArgument>, EvalError> {
        if matches!(function, ApplyFunction::Primitive(id) if id.sexptype() == r_ust_sexp::SexpType::Special)
        {
            return Ok(Vec::new());
        }
        arguments
            .iter()
            .map(|argument| {
                Ok(EvaluatedArgument {
                    name: argument.name.clone(),
                    value: self.eval_regular_argument(&argument.value)?,
                })
            })
            .collect()
    }

    pub(super) fn call_value(
        &mut self,
        value: Sexp,
        arguments: &[CallArgument],
        call: &Sexp,
    ) -> Result<EvalFlow, EvalError> {
        match value {
            Sexp::Closure { .. } => {
                let invocation = self.prepare_closure_call(value, call.clone(), arguments)?;
                self.invoke_closure(invocation)
            }
            Sexp::Primitive(id) => self.call_primitive(id, arguments, call),
            _ => Err(crate::SourceFault::type_error(
                "attempt to apply non-function".into(),
                call,
            )),
        }
    }

    fn call_primitive(
        &mut self,
        id: PrimitiveId,
        arguments: &[CallArgument],
        call: &Sexp,
    ) -> Result<EvalFlow, EvalError> {
        if id != PrimitiveId::UseMethod {
            self.visible = true;
        }
        match id {
            PrimitiveId::Syntax(id) => self.call_syntax(id, arguments, call),
            PrimitiveId::UseMethod => self.eval_use_method(arguments, call),
            PrimitiveId::OnExit => self.eval_on_exit(arguments, call),
            PrimitiveId::Return => self.eval_return(arguments, call),
            PrimitiveId::Missing => self.eval_missing(arguments),
            PrimitiveId::Quote => self.eval_quote(arguments, call),
            PrimitiveId::NamespaceExport | PrimitiveId::NamespaceInternal => {
                self.eval_namespace(arguments, call).map(EvalFlow::Value)
            }
            PrimitiveId::Expression => {
                crate::expression_values::capture(arguments).map(EvalFlow::Value)
            }
            PrimitiveId::Substitute => self.eval_substitute(arguments, call),
            PrimitiveId::Internal => self
                .eval_internal_call(arguments, call)
                .map(EvalFlow::Value),
            PrimitiveId::Call => self.construct_call(arguments, call).map(EvalFlow::Value),
            PrimitiveId::InternalEval
            | PrimitiveId::Sum
            | PrimitiveId::List
            | PrimitiveId::AsCharacter
            | PrimitiveId::Nargs
            | PrimitiveId::GlobalEnv
            | PrimitiveId::BaseEnv
            | PrimitiveId::EmptyEnv
            | PrimitiveId::IsEnvironment
            | PrimitiveId::AsCall
            | PrimitiveId::AsEnvironment
            | PrimitiveId::IsSymbol
            | PrimitiveId::IsInteger
            | PrimitiveId::AnyNa
            | PrimitiveId::Abs
            | PrimitiveId::Unclass
            | PrimitiveId::IsPairList
            | PrimitiveId::IsExpression
            | PrimitiveId::Primitive
            | PrimitiveId::Length
            | PrimitiveId::Invisible
            | PrimitiveId::Add
            | PrimitiveId::Subtract
            | PrimitiveId::Multiply
            | PrimitiveId::Divide
            | PrimitiveId::Power
            | PrimitiveId::Modulo
            | PrimitiveId::IntegerDivide
            | PrimitiveId::Colon => {
                let values = self.eval_arguments(
                    arguments,
                    !matches!(id, PrimitiveId::Length | PrimitiveId::Invisible),
                    Some(call),
                )?;
                self.call_eager_primitive(id, &values, call)
                    .map(EvalFlow::Value)
            }
        }
    }

    pub(super) fn call_eager_primitive(
        &mut self,
        id: PrimitiveId,
        arguments: &[EvaluatedArgument],
        call: &Sexp,
    ) -> Result<Sexp, EvalError> {
        match id {
            PrimitiveId::Syntax(id) => self.call_eager_syntax(id, arguments, call),
            PrimitiveId::GlobalEnv => self.environment_root(
                (id.name(), r_ust_sexp::EnvironmentRole::Global),
                arguments,
                call,
            ),
            PrimitiveId::BaseEnv => self.environment_root(
                (id.name(), r_ust_sexp::EnvironmentRole::Base),
                arguments,
                call,
            ),
            PrimitiveId::EmptyEnv => self.environment_root(
                (id.name(), r_ust_sexp::EnvironmentRole::Empty),
                arguments,
                call,
            ),
            PrimitiveId::IsEnvironment => self.is_environment(arguments, call),
            PrimitiveId::AsCall
            | PrimitiveId::AsEnvironment
            | PrimitiveId::IsSymbol
            | PrimitiveId::Unclass
            | PrimitiveId::IsPairList => self.call_invocation_primitive(id, arguments, call),
            PrimitiveId::NamespaceExport | PrimitiveId::NamespaceInternal => {
                Err(EvalError::NotImplemented(
                    "namespace selection with evaluated-only apply arguments",
                ))
            }
            PrimitiveId::IsExpression => crate::expression_values::is_expression(arguments, call),
            PrimitiveId::Expression => Err(EvalError::NotImplemented(
                "expression with evaluated-only apply arguments",
            )),
            PrimitiveId::Primitive => {
                let value = accessor::lookup(arguments, call)?;
                self.visible = true;
                Ok(value)
            }
            PrimitiveId::Add
            | PrimitiveId::Subtract
            | PrimitiveId::Multiply
            | PrimitiveId::Divide
            | PrimitiveId::Power
            | PrimitiveId::Modulo
            | PrimitiveId::IntegerDivide
            | PrimitiveId::Colon => {
                let value = self.call_arithmetic(id, arguments)?;
                self.visible = true;
                Ok(value)
            }
            PrimitiveId::Length | PrimitiveId::Invisible => {
                if arguments.len() != 1 && !(id == PrimitiveId::Invisible && arguments.is_empty()) {
                    return Err(crate::SourceFault::arity(
                        format!(
                            "{} arguments passed to '{}' which requires 1",
                            arguments.len(),
                            id.name()
                        ),
                        call,
                    ));
                }
                if let Some(name) = arguments
                    .first()
                    .and_then(|argument| argument.name.as_deref())
                    .filter(|name| *name != "x")
                {
                    return Err(crate::SourceFault::arity(
                        format!("supplied argument name '{name}' does not match 'x'"),
                        call,
                    ));
                }
                self.call_builtin(id.name(), arguments)
            }
            PrimitiveId::AnyNa => self.call_any_na(arguments, Some(call)),
            PrimitiveId::IsInteger | PrimitiveId::Abs => {
                invocation::one_argument((id.name(), arguments), call)?;
                self.call_builtin(id.name(), arguments)
            }
            PrimitiveId::InternalEval => self.internal_eval(arguments),
            PrimitiveId::AsCharacter => self.call_character_coercion(arguments, call),
            PrimitiveId::Sum | PrimitiveId::List => self.call_builtin(id.name(), arguments),
            PrimitiveId::Nargs => self.condition_nargs(arguments, call),
            PrimitiveId::Internal => Err(EvalError::Domain("illegal usage".into())),
            PrimitiveId::Substitute => Err(EvalError::NotImplemented(
                "substitute with evaluated-only apply arguments",
            )),
            PrimitiveId::OnExit => Err(EvalError::NotImplemented(
                "on.exit with evaluated-only apply arguments",
            )),
            PrimitiveId::Call
            | PrimitiveId::Return
            | PrimitiveId::Missing
            | PrimitiveId::Quote
            | PrimitiveId::UseMethod => {
                let arguments = [
                    CallArgument::positional(Sexp::Symbol("X".into())),
                    CallArgument::positional(Sexp::Symbol("...".into())),
                ];
                let EvalFlow::Value(value) = self.call_primitive(id, &arguments, call)?;
                Ok(value)
            }
        }
    }
}
