use crate::{EvalError, ListElement, RuntimeState, Sexp};

impl RuntimeState {
    pub(super) fn make_vector(&mut self, mode: &Sexp, length: &Sexp) -> Result<Sexp, EvalError> {
        let length = self.vector_length(length)?;
        let modes = super::character_values(mode)?;
        let [mode] = modes.as_slice() else {
            return Err(EvalError::Domain("invalid 'mode' argument".into()));
        };
        let mode = mode.as_deref().unwrap_or("NA");
        match mode {
            "logical" => filled(length, Some(false)).map(Sexp::Logical),
            "integer" => filled(length, Some(0)).map(Sexp::Integer),
            "numeric" | "double" => filled(length, 0.0).map(Sexp::Real),
            "character" => filled(length, Some(String::new())).map(Sexp::String),
            "list" => filled(
                length,
                ListElement {
                    name: None,
                    value: Sexp::Nil,
                },
            )
            .map(Sexp::List),
            "complex" | "raw" | "expression" | "pairlist" => Err(EvalError::NotImplemented(
                "complex/raw/expression/pairlist vector storage",
            )),
            _ => Err(EvalError::Domain(format!(
                "vector: cannot make a vector of mode '{mode}'."
            ))),
        }
    }
}

fn filled<T: Clone>(length: usize, zero: T) -> Result<Vec<T>, EvalError> {
    let mut values = Vec::new();
    values
        .try_reserve_exact(length)
        .map_err(|error| EvalError::Domain(format!("cannot allocate vector: {error}")))?;
    values.resize(length, zero);
    Ok(values)
}
