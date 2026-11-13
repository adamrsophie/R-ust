mod floor;

use super::{
    compare_values, eval_in, eval_sequence, integer_arithmetic_operand, logical_and, logical_or,
    logical_values, map2_logic, numeric_values, real_to_integer, recycled_len, BinaryOp, EvalError,
    RuntimeState, Sexp,
};

impl RuntimeState {
    pub(super) fn eval_binary(
        &mut self,
        op: BinaryOp,
        operands: (&Sexp, &Sexp),
    ) -> Result<Sexp, EvalError> {
        let (left, right) = operands;
        match op {
            BinaryOp::Modulo => self.eval_floor(floor::Operation::Modulo, operands),
            BinaryOp::IntegerDivide => self.eval_floor(floor::Operation::Quotient, operands),
            BinaryOp::Add => arithmetic_value(left, right, |a, b| a + b),
            BinaryOp::Subtract => arithmetic_value(left, right, |a, b| a - b),
            BinaryOp::Multiply => arithmetic_value(left, right, |a, b| a * b),
            BinaryOp::Divide => arithmetic(left, right, |a, b| a / b).map(Sexp::Real),
            BinaryOp::Power => arithmetic(left, right, r_ust_nmath::R_pow).map(Sexp::Real),
            BinaryOp::Sequence => eval_sequence(left, right),
            BinaryOp::And => {
                map2_logic(&logical_values(left)?, &logical_values(right)?, logical_and)
                    .map(Sexp::Logical)
            }
            BinaryOp::Or => map2_logic(&logical_values(left)?, &logical_values(right)?, logical_or)
                .map(Sexp::Logical),
            BinaryOp::AndScalar | BinaryOp::OrScalar => {
                let and = op == BinaryOp::AndScalar;
                let left = logical_operand(left, and, "x")?;
                let right = logical_operand(right, and, "y")?;
                let value = if and {
                    logical_and(left, right)
                } else {
                    logical_or(left, right)
                };
                Ok(Sexp::Logical(vec![value]))
            }
            BinaryOp::In => eval_in(left, right),
            BinaryOp::Less
            | BinaryOp::LessEqual
            | BinaryOp::Greater
            | BinaryOp::GreaterEqual
            | BinaryOp::Equal
            | BinaryOp::NotEqual => compare_values(op, left, right),
        }
    }
}

fn arithmetic_value(
    left: &Sexp,
    right: &Sexp,
    operation: impl Fn(f64, f64) -> f64,
) -> Result<Sexp, EvalError> {
    let values = arithmetic(left, right, operation)?;
    if integer_arithmetic_operand(left) && integer_arithmetic_operand(right) {
        // shortcut: overflow produces NA silently until condition warnings exist.
        Ok(Sexp::Integer(
            values.into_iter().map(real_to_integer).collect(),
        ))
    } else {
        Ok(Sexp::Real(values))
    }
}

fn arithmetic(
    left: &Sexp,
    right: &Sexp,
    f: impl Fn(f64, f64) -> f64,
) -> Result<Vec<f64>, EvalError> {
    let x = numeric_values(left)?;
    let y = numeric_values(right)?;
    Ok(x.iter()
        .cycle()
        .zip(y.iter().cycle())
        .take(recycled_len(&[x.len(), y.len()]))
        .map(|(a, b)| f(*a, *b))
        .collect())
}

pub(super) fn logical_operand(
    value: &Sexp,
    and: bool,
    side: &str,
) -> Result<Option<bool>, EvalError> {
    let operator = if and { "&&" } else { "||" };
    let invalid = || EvalError::Type(format!("invalid '{side}' type in 'x {operator} y'"));
    if matches!(value, Sexp::Nil) {
        return Err(invalid());
    }
    let values = logical_values(value).map_err(|error| match error {
        EvalError::Type(_) => invalid(),
        other => other,
    })?;
    match values.as_slice() {
        [] => Ok(None),
        [value] => Ok(*value),
        _ => Err(EvalError::Domain(format!(
            "'length = {}' in coercion to 'logical(1)'",
            values.len()
        ))),
    }
}
