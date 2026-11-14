// R 4.6.0 src/main/unique.c: Copyright (C) 1997-2025 The R Core Team;
// Copyright (C) 1995, 1996 Robert Gentleman and Ross Ihaka.
// Distributed under GPL-2.0-or-later, without any warranty.
use crate::{EvalError, EvaluatedArgument, RuntimeState, Sexp};

impl RuntimeState {
    pub(super) fn partial_match(
        &mut self,
        args: &[EvaluatedArgument],
        character: bool,
    ) -> Result<Sexp, EvalError> {
        // pmatch coerces its controls before checking character inputs; charmatch does the reverse.
        if character
            && !matches!(
                (&args[0].value, &args[1].value),
                (
                    Sexp::String(_) | Sexp::AttributedCharacter { .. },
                    Sexp::String(_) | Sexp::AttributedCharacter { .. }
                )
            )
        {
            return Err(EvalError::Type("argument is not of mode character".into()));
        }
        let nomatch = self.scalar_integer_with_warning(&args[2].value)?;
        let duplicates = if character {
            true
        } else {
            super::as_logical(&args[3].value)
                .ok_or_else(|| EvalError::Domain("invalid 'duplicates.ok' argument".into()))?
        };
        let (Some(input), Some(table)) = (
            args[0].value.character_values(),
            args[1].value.character_values(),
        ) else {
            return Err(EvalError::Type("argument is not of mode character".into()));
        };
        i32::try_from(table.len())
            .map_err(|_| EvalError::Domain("long vectors not supported yet".into()))?;
        let mut used = vec![false; table.len()];
        let mut positions = vec![None; input.len()];
        if !character {
            // R unique.c switches exact matching to first-entry hashing above this threshold.
            let hashed = input.len() > 100 && table.len() > 100;
            for (slot, value) in positions.iter_mut().zip(input) {
                let text = value.as_deref().unwrap_or("NA");
                if text.is_empty() {
                    continue;
                }
                let found = if hashed {
                    table
                        .iter()
                        .position(|target| target == value)
                        .filter(|index| duplicates || !used[*index])
                } else {
                    table.iter().enumerate().position(|(index, target)| {
                        (duplicates || !used[index]) && target.as_deref().unwrap_or("NA") == text
                    })
                };
                if let Some(index) = found {
                    *slot = Some(index);
                    used[index] = true;
                }
            }
        }
        let mut result = Vec::with_capacity(input.len());
        for (value, exact) in input.iter().zip(positions) {
            let text = value.as_deref().unwrap_or("NA");
            if let Some(index) = exact {
                result
                    .push(Some(i32::try_from(index + 1).map_err(|_| {
                        EvalError::Domain("match index overflow".into())
                    })?));
                continue;
            }
            let mut found = None;
            let mut ambiguous = false;
            let mut perfect = false;
            if character || !text.is_empty() {
                for (index, target) in table.iter().enumerate() {
                    if !duplicates && used[index] {
                        continue;
                    }
                    let target = target.as_deref().unwrap_or("NA");
                    if !target.starts_with(text) {
                        continue;
                    }
                    if character && target == text {
                        if perfect {
                            ambiguous = true;
                        } else {
                            perfect = true;
                            ambiguous = false;
                            found = Some(index);
                        }
                    } else if !perfect {
                        if found.is_some() {
                            ambiguous = true;
                        } else {
                            found = Some(index);
                        }
                    }
                }
            }
            result.push(match (found, ambiguous) {
                (_, true) => {
                    if character {
                        Some(0)
                    } else {
                        nomatch
                    }
                }
                (Some(index), false) => {
                    used[index] = true;
                    Some(
                        i32::try_from(index + 1)
                            .map_err(|_| EvalError::Domain("match index overflow".into()))?,
                    )
                }
                (None, false) => nomatch,
            });
        }
        Ok(Sexp::Integer(result))
    }
}
