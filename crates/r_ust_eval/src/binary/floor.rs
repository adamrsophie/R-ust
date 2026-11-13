mod shape;

use super::super::{
    integer_arithmetic_operand, numeric_values, real_to_integer, recycled_len, value_len,
    EvalError, RuntimeState, Sexp,
};

#[derive(Clone, Copy)]
pub(super) enum Operation {
    Modulo,
    Quotient,
}

impl Operation {
    const fn symbol(self) -> &'static str {
        match self {
            Self::Modulo => "%%",
            Self::Quotient => "%/%",
        }
    }

    fn scalar(self, x: f64, y: f64) -> (f64, bool) {
        let q = x / y;
        let opposite = (x < 0.0 && y > 0.0) || (x > 0.0 && y < 0.0);
        match self {
            Self::Modulo => {
                if y == 0.0 {
                    return (f64::NAN, false);
                }
                if y.abs() * f64::EPSILON > 1.0 && x.is_finite() && x.abs() <= y.abs() {
                    let value = if x.abs() == y.abs() {
                        0.0
                    } else if opposite {
                        x + y
                    } else {
                        x
                    };
                    return (value, false);
                }
                let loss = q.is_finite() && q.abs() * f64::EPSILON > 1.0;
                // R 4.6.0 arm64 contracts both residuals into fused multiply-subtract.
                let residual = (-q.floor()).mul_add(y, x);
                ((-(residual / y).floor()).mul_add(y, residual), loss)
            }
            Self::Quotient => {
                if y == 0.0 || q.abs() * f64::EPSILON > 1.0 || !q.is_finite() {
                    return (q, false);
                }
                if q.abs() < 1.0 {
                    return (if q < 0.0 || opposite { -1.0 } else { 0.0 }, false);
                }
                let residual = (-q.floor()).mul_add(y, x);
                (q.floor() + (residual / y).floor(), false)
            }
        }
    }
}

impl RuntimeState {
    pub(super) fn eval_floor(
        &mut self,
        op: Operation,
        operands: (&Sexp, &Sexp),
    ) -> Result<Sexp, EvalError> {
        let (left, right) = operands;
        // shortcut: mixed factor methods await S3 group dispatch; reject instead of using factor codes.
        if let (Sexp::Factor { ordered: a, .. }, Sexp::Factor { ordered: b, .. }) = operands {
            if a != b {
                return Err(EvalError::NotImplemented(
                    "mixed ordered and unordered factor arithmetic",
                ));
            }
        }
        let factor = match (left, right) {
            (Sexp::Factor { ordered, .. }, _) | (_, Sexp::Factor { ordered, .. }) => Some(*ordered),
            _ => None,
        };
        if let Some(ordered) = factor {
            let message = if ordered {
                format!("'{}' is not meaningful for ordered factors", op.symbol())
            } else {
                format!("'{}' not meaningful for factors", op.symbol())
            };
            self.record_parse_warnings(&[message])?;
            return Ok(Sexp::Logical(vec![
                None;
                value_len(left).max(value_len(right))
            ]));
        }
        let x = numeric_values(left)?;
        let y = numeric_values(right)?;
        let dims = self.floor_shape(operands)?;
        let length = recycled_len(&[x.len(), y.len()]);
        if length > 0 && !length.is_multiple_of(x.len().min(y.len())) {
            self.record_parse_warnings(&[
                "longer object length is not a multiple of shorter object length".into(),
            ])?;
        }
        let integer = integer_arithmetic_operand(left) && integer_arithmetic_operand(right);
        let mut values = Vec::with_capacity(length);
        for index in 0..length {
            let (value, loss) = op.scalar(x[index % x.len()], y[index % y.len()]);
            if loss && !integer {
                self.record_parse_warnings(&[
                    "probable complete loss of accuracy in modulus".into()
                ])?;
            }
            values.push(value);
        }
        let data = if integer {
            Sexp::Integer(values.into_iter().map(real_to_integer).collect())
        } else {
            Sexp::Real(values)
        };
        shape::attach(data, dims)
    }
}
