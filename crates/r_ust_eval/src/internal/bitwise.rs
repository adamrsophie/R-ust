use std::borrow::Cow;

use crate::{typeof_value, EvalError, EvaluatedArgument, RuntimeState, Sexp};

#[derive(Clone, Copy)]
pub(super) enum Binary {
    And,
    Or,
    Xor,
    ShiftL,
    ShiftR,
}

impl Binary {
    fn name(self) -> &'static str {
        match self {
            Self::And => "bitwAnd",
            Self::Or => "bitwOr",
            Self::Xor => "bitwXor",
            Self::ShiftL => "bitShiftL",
            Self::ShiftR => "bitShiftR",
        }
    }

    fn apply(self, left: i32, right: i32) -> Option<i32> {
        let value = match self {
            Self::And => left & right,
            Self::Or => left | right,
            Self::Xor => left ^ right,
            Self::ShiftL => {
                let count = u32::try_from(right).ok()?;
                let bits = u32::from_ne_bytes(left.to_ne_bytes());
                let shifted = bits.checked_shl(count)?;
                i32::from_ne_bytes(shifted.to_ne_bytes())
            }
            Self::ShiftR => {
                let count = u32::try_from(right).ok()?;
                let bits = u32::from_ne_bytes(left.to_ne_bytes());
                let shifted = bits.checked_shr(count)?;
                i32::from_ne_bytes(shifted.to_ne_bytes())
            }
        };
        present(value)
    }
}

fn present(value: i32) -> Option<i32> {
    (value != i32::MIN).then_some(value)
}

fn data(value: &Sexp) -> &Sexp {
    let mut value = value;
    while let Sexp::Matrix { data, .. } = value {
        value = data;
    }
    value
}

fn integers<'a>(value: &'a Sexp, name: &str) -> Result<Cow<'a, [Option<i32>]>, EvalError> {
    match data(value) {
        Sexp::Integer(values) => Ok(Cow::Borrowed(values)),
        Sexp::Factor { codes, .. } => codes
            .iter()
            .map(|code| {
                code.map(i32::try_from)
                    .transpose()
                    .map_err(|_| EvalError::Type("factor code exceeds R integer range".into()))
            })
            .collect::<Result<Vec<_>, _>>()
            .map(Cow::Owned),
        other => Err(EvalError::Type(format!(
            "unimplemented type '{}' in '{name}'\n",
            typeof_value(other)
        ))),
    }
}

impl RuntimeState {
    fn bitwise_integer(&mut self, value: &Sexp) -> Result<Sexp, EvalError> {
        if !matches!(
            data(value),
            Sexp::Nil
                | Sexp::Logical(_)
                | Sexp::Integer(_)
                | Sexp::Real(_)
                | Sexp::String(_)
                | Sexp::AttributedCharacter { .. }
                | Sexp::Vector(_)
                | Sexp::ClassedList { .. }
                | Sexp::List(_)
                | Sexp::ListObject(_)
                | Sexp::Factor { .. }
                | Sexp::DataFrame { .. }
        ) {
            return Err(EvalError::Type(format!(
                "cannot coerce type '{}' to vector of type 'integer'",
                typeof_value(value)
            )));
        }
        self.call_numeric_coercion(
            "as.integer",
            &[EvaluatedArgument {
                name: None,
                value: value.clone(),
            }],
        )
    }

    fn bitwise_operand<'a>(&mut self, value: &'a Sexp) -> Result<Cow<'a, Sexp>, EvalError> {
        if matches!(data(value), Sexp::Real(_)) {
            self.bitwise_integer(value).map(Cow::Owned)
        } else {
            Ok(Cow::Borrowed(value))
        }
    }

    pub(super) fn bitwise_not(&mut self, value: &Sexp) -> Result<Sexp, EvalError> {
        let value = self.bitwise_operand(value)?;
        let values = integers(&value, "bitwNot")?;
        Ok(Sexp::Integer(
            values
                .iter()
                .map(|value| value.and_then(present).and_then(|value| present(!value)))
                .collect(),
        ))
    }

    pub(super) fn bitwise_binary(
        &mut self,
        operation: Binary,
        args: &[EvaluatedArgument],
    ) -> Result<Sexp, EvalError> {
        let left = self.bitwise_operand(&args[0].value)?;
        let right = match operation {
            Binary::ShiftL | Binary::ShiftR => {
                if matches!(data(&args[1].value), Sexp::Integer(_) | Sexp::Factor { .. }) {
                    Cow::Borrowed(&args[1].value)
                } else {
                    Cow::Owned(self.bitwise_integer(&args[1].value)?)
                }
            }
            Binary::And | Binary::Or | Binary::Xor => self.bitwise_operand(&args[1].value)?,
        };
        if typeof_value(&left) != typeof_value(&right) {
            return Err(EvalError::Type(
                "'a' and 'b' must have the same type".into(),
            ));
        }
        let left = integers(&left, operation.name())?;
        let right = integers(&right, operation.name())?;
        let length = if left.is_empty() || right.is_empty() {
            0
        } else {
            left.len().max(right.len())
        };
        Ok(Sexp::Integer(
            (0..length)
                .map(|index| {
                    let a = present(left[index % left.len()]?)?;
                    let b = present(right[index % right.len()]?)?;
                    operation.apply(a, b)
                })
                .collect(),
        ))
    }
}
