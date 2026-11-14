mod floor;

use super::{
    colon_sequence, compare_values, eval_in, integer_arithmetic_operand, logical_and, logical_or,
    logical_values, map2_logic, numeric_values, real_to_integer, recycled_len, value_len, BinaryOp,
    EvalError, RuntimeState, Sexp,
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
            BinaryOp::Add => self.arithmetic(operands, (|a, b| a + b, true)),
            BinaryOp::Subtract => self.arithmetic(operands, (|a, b| a - b, true)),
            BinaryOp::Multiply => self.arithmetic(operands, (|a, b| a * b, true)),
            BinaryOp::Divide => self.arithmetic(operands, (|a, b| a / b, false)),
            BinaryOp::Power => self.arithmetic(operands, (r_ust_nmath::R_pow, false)),
            BinaryOp::Sequence => self.eval_sequence(operands),
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

    fn eval_sequence(&mut self, operands: (&Sexp, &Sexp)) -> Result<Sexp, EvalError> {
        let (left, right) = operands;
        let left_len = value_len(left);
        let right_len = value_len(right);
        if left_len == 0 || right_len == 0 {
            return Err(EvalError::Domain("argument of length 0".into()));
        }
        let length = if left_len > 1 { left_len } else { right_len };
        if length > 1 {
            self.record_parse_warnings(&[format!(
                "numerical expression has {length} elements: only the first used"
            )])?;
        }
        let from = self.scalar_real_with_warning(left)?;
        let to = self.scalar_real_with_warning(right)?;
        if from.is_nan() || to.is_nan() {
            return Err(EvalError::Domain("NA/NaN argument".into()));
        }
        colon_sequence(from, to)
    }

    fn arithmetic(
        &mut self,
        operands: (&Sexp, &Sexp),
        operation: (impl Fn(f64, f64) -> f64, bool),
    ) -> Result<Sexp, EvalError> {
        let (left, right) = operands;
        let x = arithmetic_numbers(left)?;
        let y = arithmetic_numbers(right)?;
        let length = recycled_len(&[x.len(), y.len()]);
        if length > 0 && !length.is_multiple_of(x.len().min(y.len())) {
            self.record_parse_warnings(&[
                "longer object length is not a multiple of shorter object length".into(),
            ])?;
        }
        let integer =
            operation.1 && integer_arithmetic_operand(left) && integer_arithmetic_operand(right);
        let values: Vec<_> = x
            .iter()
            .cycle()
            .zip(y.iter().cycle())
            .take(length)
            .map(|(a, b)| operation.0(*a, *b))
            .collect();
        if integer {
            if values
                .iter()
                .any(|value| !value.is_nan() && real_to_integer(*value).is_none())
            {
                self.record_parse_warnings(&["NAs produced by integer overflow".into()])?;
            }
            Ok(Sexp::Integer(
                values.into_iter().map(real_to_integer).collect(),
            ))
        } else {
            Ok(Sexp::Real(values))
        }
    }
}

fn arithmetic_numbers(value: &Sexp) -> Result<Vec<f64>, EvalError> {
    numeric_values(value).map_err(|error| match error {
        EvalError::Type(_) => EvalError::Type("non-numeric argument to binary operator".into()),
        other => other,
    })
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
