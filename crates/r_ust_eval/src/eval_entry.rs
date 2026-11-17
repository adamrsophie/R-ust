use crate::{
    base, character_attributes, internal, syntax_raw, EvalError, EvalFlow, RuntimeState, Sexp,
};

impl RuntimeState {
    pub(super) fn eval_flow_body(&mut self, expr: &Sexp) -> Result<EvalFlow, EvalError> {
        character_attributes::validate(expr)?;
        base::initialize(&self.current)?;
        self.visible = true;
        if let Sexp::Managed(value) = expr {
            let borrowed = value.borrow()?;
            if matches!(
                borrowed.value().sexptype(),
                r_ust_sexp::SexpType::Symbol
                    | r_ust_sexp::SexpType::Language
                    | r_ust_sexp::SexpType::Promise
                    | r_ust_sexp::SexpType::Bytecode
                    | r_ust_sexp::SexpType::Char
            ) {
                return self.eval_flow_body(borrowed.value());
            }
            return Ok(EvalFlow::Value(expr.clone()));
        }
        if let Some(Sexp::Call {
            function,
            arguments,
        }) = syntax_raw::call(expr)
        {
            return self.eval_call(&function, &arguments);
        }
        let result = match expr {
            Sexp::Bytecode(code) => self.eval_bytecode(code)?,
            Sexp::Nil
            | Sexp::AttributedAtomic { .. }
            | Sexp::AttributedAggregate { .. }
            | Sexp::ListObject(_)
            | Sexp::PairListObject(_)
            | Sexp::Environment(_)
            | Sexp::Primitive(_)
            | Sexp::Logical(_)
            | Sexp::Raw(_)
            | Sexp::Integer(_)
            | Sexp::Real(_)
            | Sexp::String(_)
            | Sexp::AttributedCharacter { .. }
            | Sexp::Expression { .. }
            | Sexp::Vector(_)
            | Sexp::NamedList { .. }
            | Sexp::ClassedList { .. }
            | Sexp::List(_)
            | Sexp::PairList(_)
            | Sexp::MissingValue
            | Sexp::Factor { .. }
            | Sexp::DataFrame { .. }
            | Sexp::Matrix { .. }
            | Sexp::Dots(_)
            | Sexp::Closure { .. } => EvalFlow::Value(expr.clone()),
            Sexp::InternalCall {
                operation,
                arguments,
            } => EvalFlow::Value(self.eval_internal(
                *operation,
                arguments,
                internal::InternalSite {
                    outer: &Sexp::Nil,
                    inner: None,
                },
            )?),
            Sexp::MissingArg(name) => return Err(self.evaluated_missing(name)),
            Sexp::Promise { .. } => EvalFlow::Value(self.force_value(expr.clone())?),
            Sexp::Symbol(name) => self.eval_symbol(name)?,
            Sexp::Binary { op, left, right } => {
                return self.eval_binary_expression(*op, (left, right));
            }
            Sexp::Unary { op, value } => {
                return self.eval_unary_expression(*op, value);
            }
            Sexp::Call {
                function,
                arguments,
            } => self.eval_call(function, arguments)?,
            Sexp::Parenthesized(value) => return self.eval_parenthesized(value),
            _ => return self.eval_legacy_expression(expr),
        };
        let EvalFlow::Value(value) = &result;
        character_attributes::validate(value)?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests;
