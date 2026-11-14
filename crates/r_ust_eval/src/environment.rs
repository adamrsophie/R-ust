use super::{CallArgument, EvalError, EvalFlow, RuntimeState, Sexp};
use r_ust_sexp::{EnvironmentRole, PrimitiveId};

#[derive(Clone, Debug, PartialEq)]
pub(super) enum ApplyFunction {
    Closure(Sexp),
    Primitive(PrimitiveId),
    Builtin(String),
}

pub(super) fn has_builtin(name: &str) -> bool {
    super::is_builtin(name) || matches!(name, ".Internal" | "...length")
}

impl RuntimeState {
    pub(super) fn lookup_function(
        &mut self,
        name: &str,
        builtin: bool,
    ) -> Result<Option<ApplyFunction>, EvalError> {
        self.search_function(name, builtin, true)
    }

    fn search_function(
        &mut self,
        name: &str,
        builtin: bool,
        skip_nonfunctions: bool,
    ) -> Result<Option<ApplyFunction>, EvalError> {
        super::base::initialize(&self.current)?;
        let mut current = Some(self.current.clone());
        while let Some(frame) = current {
            if let Some(value) = frame.get_local(name)? {
                match self.force_value(value)? {
                    Sexp::MissingValue => return Err(EvalError::MissingArgument(name.into())),
                    value @ Sexp::Closure { .. } => return Ok(Some(ApplyFunction::Closure(value))),
                    Sexp::Primitive(id) => return Ok(Some(ApplyFunction::Primitive(id))),
                    _ if !skip_nonfunctions => {
                        return Err(EvalError::Type("attempt to apply non-function".into()))
                    }
                    _ => {}
                }
            } else if builtin
                && matches!(
                    frame.role()?,
                    EnvironmentRole::Base | EnvironmentRole::BaseNamespace
                )
            {
                return Ok(Some(ApplyFunction::Builtin(name.into())));
            }
            current = frame.parent()?;
        }
        Ok(None)
    }

    pub(super) fn require_function(
        &mut self,
        name: &str,
        builtin: bool,
    ) -> Result<ApplyFunction, EvalError> {
        self.lookup_function(name, builtin)?
            .ok_or_else(|| EvalError::UndefinedFunction(name.into()))
    }

    pub(super) fn lookup(&mut self, name: &str) -> Result<Option<Sexp>, EvalError> {
        if let Some(index) = name
            .strip_prefix("..")
            .and_then(|s| s.parse::<usize>().ok())
        {
            return self.lookup_dot(index).map(Some);
        }
        match self.raw_lookup(name)? {
            Some(Sexp::MissingValue) => Err(EvalError::MissingArgument(name.into())),
            Some(value) => self.force_value(value).map(Some),
            None => Ok(None),
        }
    }

    pub(super) fn raw_lookup(&self, name: &str) -> Result<Option<Sexp>, EvalError> {
        super::base::initialize(&self.current)?;
        Ok(self.current.get(name)?)
    }

    pub(super) fn resolve_apply_function(
        &mut self,
        expr: &Sexp,
    ) -> Result<ApplyFunction, EvalError> {
        if let Sexp::Parenthesized(inner) = expr {
            match self.require_function("(", true)? {
                ApplyFunction::Builtin(_) => return self.resolve_parenthesized_function(inner),
                function => {
                    let EvalFlow::Value(value) = self.call_resolved(
                        function,
                        &[CallArgument::positional(inner.as_ref().clone())],
                    )?;
                    return callable(value);
                }
            }
        }
        if let Sexp::Symbol(name) = expr {
            return self.require_function(name, has_builtin(name));
        }
        callable(self.eval_regular_argument(expr)?)
    }

    pub(super) fn resolve_parenthesized_function(
        &mut self,
        expr: &Sexp,
    ) -> Result<ApplyFunction, EvalError> {
        if let Sexp::Parenthesized(inner) = expr {
            return match self.require_function("(", true)? {
                ApplyFunction::Builtin(_) => self.resolve_parenthesized_function(inner),
                function => {
                    let EvalFlow::Value(value) = self.call_resolved(
                        function,
                        &[CallArgument::positional(inner.as_ref().clone())],
                    )?;
                    call_head(value)
                }
            };
        }
        if let Sexp::Symbol(name) = expr {
            return self
                .search_function(name, has_builtin(name), false)?
                .ok_or_else(|| EvalError::UndefinedSymbol(name.into()));
        }
        call_head(self.eval_regular_argument(expr)?)
    }

    pub(super) fn call_resolved(
        &mut self,
        function: ApplyFunction,
        arguments: &[CallArgument],
    ) -> Result<EvalFlow, EvalError> {
        match function {
            ApplyFunction::Closure(value) => self.call_value(value, arguments),
            ApplyFunction::Primitive(id) => self.call_value(Sexp::Primitive(id), arguments),
            ApplyFunction::Builtin(name) => self.eval_builtin_call(&name, arguments),
        }
    }

    pub(super) fn eval_parenthesized(&mut self, value: &Sexp) -> Result<EvalFlow, EvalError> {
        match self.require_function("(", true)? {
            ApplyFunction::Builtin(_) => self.eval_flow(value),
            function => self.call_resolved(function, &[CallArgument::positional(value.clone())]),
        }
    }

    pub(super) fn assign_local(&mut self, name: String, value: Sexp) -> Result<(), EvalError> {
        self.current
            .insert(name.clone(), &value)
            .map_err(|error| super::environment_members::member_graph_error(error, &name))
    }

    pub(super) fn assign_super(&mut self, name: String, value: Sexp) -> Result<(), EvalError> {
        self.current
            .assign_parent(name.clone(), &value)
            .map_err(|error| super::environment_members::member_graph_error(error, &name))
    }
}

fn callable(value: Sexp) -> Result<ApplyFunction, EvalError> {
    match value {
        Sexp::Primitive(id) => Ok(ApplyFunction::Primitive(id)),
        closure @ Sexp::Closure { .. } => Ok(ApplyFunction::Closure(closure)),
        other => Err(EvalError::Type(format!(
            "apply function must be callable, found {:?}",
            other.sexptype()
        ))),
    }
}

fn call_head(value: Sexp) -> Result<ApplyFunction, EvalError> {
    match value {
        Sexp::Primitive(id) => Ok(ApplyFunction::Primitive(id)),
        closure @ Sexp::Closure { .. } => Ok(ApplyFunction::Closure(closure)),
        _ => Err(EvalError::Type("attempt to apply non-function".into())),
    }
}
