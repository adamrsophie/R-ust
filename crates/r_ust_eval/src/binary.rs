use super::{
    compare_values, eval_in, eval_scalar_logical, eval_sequence, integer_arithmetic_operand,
    logical_and, logical_or, logical_values, map2, map2_logic, numeric_values, real_to_integer,
    BinaryOp, EvalError, Sexp,
};

pub(super) fn eval_binary(op: BinaryOp, left: &Sexp, right: &Sexp) -> Result<Sexp, EvalError> {
    match op {
        BinaryOp::Add | BinaryOp::Subtract | BinaryOp::Multiply => {
            let values = map2(
                &numeric_values(left)?,
                &numeric_values(right)?,
                |a, b| match op {
                    BinaryOp::Add => a + b,
                    BinaryOp::Subtract => a - b,
                    BinaryOp::Multiply => a * b,
                    _ => unreachable!(),
                },
            )?;
            if integer_arithmetic_operand(left) && integer_arithmetic_operand(right) {
                // shortcut: overflow produces NA silently until condition warnings exist.
                Ok(Sexp::Integer(
                    values.into_iter().map(real_to_integer).collect(),
                ))
            } else {
                Ok(Sexp::Real(values))
            }
        }
        BinaryOp::Divide => map2(&numeric_values(left)?, &numeric_values(right)?, |a, b| {
            a / b
        })
        .map(Sexp::Real),
        BinaryOp::Power => map2(
            &numeric_values(left)?,
            &numeric_values(right)?,
            r_ust_nmath::R_pow,
        )
        .map(Sexp::Real),
        BinaryOp::Sequence => eval_sequence(left, right),
        BinaryOp::And => map2_logic(&logical_values(left)?, &logical_values(right)?, logical_and)
            .map(Sexp::Logical),
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
