mod floor;

use super::{
    compare_values, eval_in, eval_scalar_logical, eval_sequence, integer_arithmetic_operand,
    logical_and, logical_or, logical_values, map2_logic, numeric_values, real_to_integer,
    recycled_len, BinaryOp, EvalError, RuntimeState, Sexp,
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
            BinaryOp::Add | BinaryOp::Subtract | BinaryOp::Multiply => {
                let values = arithmetic(left, right, |a, b| match op {
                    BinaryOp::Add => a + b,
                    BinaryOp::Subtract => a - b,
                    BinaryOp::Multiply => a * b,
                    _ => unreachable!(),
                })?;
                if integer_arithmetic_operand(left) && integer_arithmetic_operand(right) {
                    // shortcut: overflow produces NA silently until condition warnings exist.
                    Ok(Sexp::Integer(
                        values.into_iter().map(real_to_integer).collect(),
                    ))
                } else {
                    Ok(Sexp::Real(values))
                }
            }
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
                eval_scalar_logical(op, left, right).map(|value| Sexp::Logical(vec![value]))
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
