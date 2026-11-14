use r_ust_sexp::PrimitiveId;

use super::{
    logical_values, numeric_values, BinaryOp, CallArgument, EvalError, EvalFlow, EvaluatedArgument,
    RuntimeState, Sexp, UnaryOp,
};

impl RuntimeState {
    pub(super) fn eval_binary_expression(
        &mut self,
        op: BinaryOp,
        operands: (&Sexp, &Sexp),
    ) -> Result<EvalFlow, EvalError> {
        let name = match op {
            BinaryOp::Add => Some("+"),
            BinaryOp::Subtract => Some("-"),
            BinaryOp::Multiply => Some("*"),
            BinaryOp::Divide => Some("/"),
            BinaryOp::Power => Some("^"),
            BinaryOp::Modulo => Some("%%"),
            BinaryOp::IntegerDivide => Some("%/%"),
            BinaryOp::Sequence => Some(":"),
            BinaryOp::And
            | BinaryOp::Or
            | BinaryOp::AndScalar
            | BinaryOp::OrScalar
            | BinaryOp::In
            | BinaryOp::Less
            | BinaryOp::LessEqual
            | BinaryOp::Greater
            | BinaryOp::GreaterEqual
            | BinaryOp::Equal
            | BinaryOp::NotEqual => None,
        };
        let (left, right) = operands;
        if let Some(name) = name {
            let function = self
                .lookup_function(name)?
                .ok_or_else(|| EvalError::UndefinedSymbol(name.into()))?;
            return self.call_value(
                function,
                &[
                    CallArgument::positional(left.clone()),
                    CallArgument::positional(right.clone()),
                ],
            );
        }
        let EvalFlow::Value(left) = self.eval_flow(left)?;
        if matches!(op, BinaryOp::AndScalar | BinaryOp::OrScalar) {
            match (
                op,
                super::binary::logical_operand(&left, op == BinaryOp::AndScalar, "x")?,
            ) {
                (BinaryOp::AndScalar, Some(false)) => {
                    self.visible = true;
                    return Ok(EvalFlow::Value(Sexp::Logical(vec![Some(false)])));
                }
                (BinaryOp::OrScalar, Some(true)) => {
                    self.visible = true;
                    return Ok(EvalFlow::Value(Sexp::Logical(vec![Some(true)])));
                }
                _ => {}
            }
        }
        let EvalFlow::Value(right) = self.eval_flow(right)?;
        let result = self.eval_binary(op, (&left, &right))?;
        self.visible = true;
        Ok(EvalFlow::Value(result))
    }

    pub(super) fn eval_unary_expression(
        &mut self,
        op: UnaryOp,
        value: &Sexp,
    ) -> Result<EvalFlow, EvalError> {
        let name = match op {
            UnaryOp::Plus => "+",
            UnaryOp::Minus => "-",
            UnaryOp::Not => {
                let EvalFlow::Value(value) = self.eval_flow(value)?;
                let result = eval_unary(op, &value)?;
                self.visible = true;
                return Ok(EvalFlow::Value(result));
            }
        };
        let function = self
            .lookup_function(name)?
            .ok_or_else(|| EvalError::UndefinedSymbol(name.into()))?;
        self.call_value(function, &[CallArgument::positional(value.clone())])
    }

    pub(super) fn call_arithmetic(
        &mut self,
        id: PrimitiveId,
        arguments: &[EvaluatedArgument],
    ) -> Result<Sexp, EvalError> {
        let op = match id {
            PrimitiveId::Add => BinaryOp::Add,
            PrimitiveId::Subtract => BinaryOp::Subtract,
            PrimitiveId::Multiply => BinaryOp::Multiply,
            PrimitiveId::Divide => BinaryOp::Divide,
            PrimitiveId::Power => BinaryOp::Power,
            PrimitiveId::Modulo => BinaryOp::Modulo,
            PrimitiveId::IntegerDivide => BinaryOp::IntegerDivide,
            PrimitiveId::Colon => BinaryOp::Sequence,
            PrimitiveId::Sum
            | PrimitiveId::Primitive
            | PrimitiveId::Length
            | PrimitiveId::Invisible
            | PrimitiveId::Return
            | PrimitiveId::Missing
            | PrimitiveId::Quote => {
                return Err(EvalError::NotImplemented("non-arithmetic primitive"))
            }
        };
        if id == PrimitiveId::Colon && arguments.len() != 2 {
            return Err(EvalError::Arity(format!(
                "{} argument{} passed to ':' which requires 2",
                arguments.len(),
                if arguments.len() == 1 { "" } else { "s" }
            )));
        }
        match arguments {
            [left, right] => self.eval_binary(op, (&left.value, &right.value)),
            [value] => match id {
                PrimitiveId::Add => eval_unary(UnaryOp::Plus, &value.value),
                PrimitiveId::Subtract => eval_unary(UnaryOp::Minus, &value.value),
                PrimitiveId::Multiply
                | PrimitiveId::Divide
                | PrimitiveId::Power
                | PrimitiveId::Modulo
                | PrimitiveId::IntegerDivide => {
                    unary_numbers(&value.value)?;
                    Err(EvalError::Type("invalid unary operator".into()))
                }
                PrimitiveId::Colon
                | PrimitiveId::Sum
                | PrimitiveId::Primitive
                | PrimitiveId::Length
                | PrimitiveId::Invisible
                | PrimitiveId::Return
                | PrimitiveId::Missing
                | PrimitiveId::Quote => Err(EvalError::NotImplemented("non-unary primitive")),
            },
            _ => Err(EvalError::Arity(
                "operator needs one or two arguments".into(),
            )),
        }
    }
}

fn unary_numbers(value: &Sexp) -> Result<Vec<f64>, EvalError> {
    if matches!(value, Sexp::Nil) {
        return Err(EvalError::Type("invalid argument to unary operator".into()));
    }
    numeric_values(value).map_err(|error| match error {
        EvalError::Type(_) => EvalError::Type("invalid argument to unary operator".into()),
        other => other,
    })
}

fn eval_unary(op: UnaryOp, value: &Sexp) -> Result<Sexp, EvalError> {
    if op == UnaryOp::Not && matches!(value, Sexp::Environment(_)) {
        return Err(EvalError::Type("invalid argument type".into()));
    }
    if matches!(op, UnaryOp::Plus | UnaryOp::Minus) {
        let integers = match value {
            Sexp::Logical(values) => {
                Some(values.iter().map(|value| value.map(i32::from)).collect())
            }
            Sexp::Integer(values) => Some(values.clone()),
            _ => None,
        };
        if let Some(values) = integers {
            return Ok(Sexp::Integer(if op == UnaryOp::Minus {
                values
                    .into_iter()
                    .map(|value| value.and_then(i32::checked_neg))
                    .collect()
            } else {
                values
            }));
        }
    }
    match op {
        UnaryOp::Plus => Ok(Sexp::Real(unary_numbers(value)?)),
        UnaryOp::Minus => Ok(Sexp::Real(
            unary_numbers(value)?
                .into_iter()
                .map(|value| -value)
                .collect(),
        )),
        UnaryOp::Not => Ok(Sexp::Logical(
            logical_values(value)?
                .into_iter()
                .map(|value| value.map(|value| !value))
                .collect(),
        )),
    }
}
