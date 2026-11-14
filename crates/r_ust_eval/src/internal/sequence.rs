use crate::{EvalError, EvaluatedArgument, RuntimeState, Sexp};

// R 4.6.0 src/main/seq.c do_sequence, GPL-2.0-or-later.
// Copyright (C) 1998-2026 The R Core Team.
// Copyright (C) 1995-1998 Robert Gentleman and Ross Ihaka.
impl RuntimeState {
    pub(super) fn sequence(&mut self, args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
        let lengths = integer_values(&args[0].value, "nvec")?;
        let from = integer_values(&args[1].value, "from")?;
        let by = integer_values(&args[2].value, "by")?;
        let recycle_value = &args[3].value;
        let recycle_length = crate::value_len(recycle_value)?;
        if recycle_length > 1
            && matches!(
                recycle_value,
                Sexp::Logical(_)
                    | Sexp::Integer(_)
                    | Sexp::Real(_)
                    | Sexp::String(_)
                    | Sexp::AttributedCharacter { .. }
                    | Sexp::Factor { .. }
                    | Sexp::Matrix { .. }
            )
        {
            return Err(EvalError::Domain(format!(
                "'length = {recycle_length}' in coercion to 'logical(1)'"
            )));
        }
        let recycle = super::as_logical(recycle_value)
            .ok_or_else(|| EvalError::Domain("NA in coercion to boolean".into()))?;
        if lengths.is_empty() {
            return Ok(Sexp::Integer(Vec::new()));
        }
        for (values, name) in [(from, "from"), (by, "by")] {
            if values.is_empty() {
                if recycle {
                    return Ok(Sexp::Integer(Vec::new()));
                }
                return Err(EvalError::Domain(format!(
                    "'{name}' has length 0, but not 'nvec'; 'recycle = TRUE' returns empty here"
                )));
            }
        }
        let mut count = lengths.len().max(from.len()).max(by.len());
        if !recycle && lengths.len() < count {
            if !self.warned_sequence_recycle && integer_values(recycle_value, "recycle").is_ok() {
                self.record_warnings_at(&[format!(
                    "length(nvec) {} < {count} = max(length(from), length(by)) -- future R`s default 'recycle = TRUE' will recycle 'nvec'",
                    lengths.len()
                )], Some(&self.source_current_call()))?;
                self.warned_sequence_recycle = true;
            }
            count = lengths.len();
        }
        let invalid_length =
            || EvalError::Domain("'nvec' must be a vector of non-negative integers".into());
        let mut total = 0usize;
        for index in 0..count {
            let length = lengths[index % lengths.len()].ok_or_else(invalid_length)?;
            let length = usize::try_from(length).map_err(|_| invalid_length())?;
            total = total
                .checked_add(length)
                .ok_or_else(|| EvalError::Domain("vector size specified is too large".into()))?;
        }
        if u64::try_from(total).map_or(true, |length| length > 4_503_599_627_370_496) {
            return Err(EvalError::Domain(
                "vector size specified is too large".into(),
            ));
        }
        let mut result = Vec::new();
        result
            .try_reserve_exact(total)
            .map_err(|error| EvalError::Domain(format!("cannot allocate vector: {error}")))?;
        for index in 0..count {
            let length = lengths[index % lengths.len()].ok_or_else(invalid_length)?;
            let start = from[index % from.len()];
            let step = by[index % by.len()];
            if length != 0 && start.is_none() {
                return Err(EvalError::Domain("'from' contains NAs".into()));
            }
            if length >= 2 && step.is_none() {
                return Err(EvalError::Domain("'by' contains NAs".into()));
            }
            let mut value = start.unwrap_or(i32::MIN);
            for _ in 0..length {
                result.push((value != i32::MIN).then_some(value));
                value = value.wrapping_add(step.unwrap_or(i32::MIN));
            }
        }
        Ok(Sexp::Integer(result))
    }
}

fn integer_values<'a>(value: &'a Sexp, name: &str) -> Result<&'a [Option<i32>], EvalError> {
    let value = match value {
        Sexp::Matrix { data, .. } => data.as_ref(),
        value => value,
    };
    let Sexp::Integer(values) = value else {
        return Err(EvalError::Domain(format!(
            "'{name}' is not of mode integer"
        )));
    };
    Ok(values)
}
