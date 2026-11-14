use super::{value_len, EvalError, RuntimeState, Sexp};

impl RuntimeState {
    pub(super) fn floor_shape(
        &mut self,
        operands: (&Sexp, &Sexp),
    ) -> Result<Option<(usize, usize)>, EvalError> {
        let (left, right) = operands;
        match (left, right) {
            (
                Sexp::Matrix { nrow, ncol, .. },
                Sexp::Matrix {
                    nrow: other_rows,
                    ncol: other_cols,
                    ..
                },
            ) => {
                if (nrow, ncol) != (other_rows, other_cols) {
                    return Err(EvalError::Domain("non-conformable arrays".into()));
                }
                Ok(Some((*nrow, *ncol)))
            }
            (matrix @ Sexp::Matrix { nrow, ncol, .. }, vector)
            | (vector, matrix @ Sexp::Matrix { nrow, ncol, .. }) => {
                let (array_length, vector_length) = (value_len(matrix)?, value_len(vector)?);
                if array_length == 1 && vector_length != 1 {
                    if vector_length > 0 {
                        let order = if matches!(left, Sexp::Matrix { .. }) {
                            "array-vector"
                        } else {
                            "vector-array"
                        };
                        self.record_parse_warnings(&[format!("Recycling array of length 1 in {order} arithmetic is deprecated.\n  Use c() or as.vector() instead.")])?;
                    }
                    Ok(None)
                } else if array_length > 0 && vector_length == 0 {
                    Ok(None)
                } else {
                    Ok(Some((*nrow, *ncol)))
                }
            }
            _ => Ok(None),
        }
    }
}

pub(super) fn attach(data: Sexp, dims: Option<(usize, usize)>) -> Result<Sexp, EvalError> {
    match dims {
        Some((nrow, ncol)) => {
            let product = nrow
                .checked_mul(ncol)
                .ok_or_else(|| EvalError::Domain("matrix dimensions overflow".into()))?;
            let length = value_len(&data)?;
            if product != length {
                return Err(EvalError::Domain(format!(
                    "dims [product {product}] do not match the length of object [{length}]"
                )));
            }
            Ok(Sexp::Matrix {
                data: Box::new(data),
                nrow,
                ncol,
            })
        }
        None => Ok(data),
    }
}
