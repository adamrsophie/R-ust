//! R 4.6.0 rowsum kernels; row names and general S3 attributes are not represented.
//! shortcut: character ordering uses C collation, add locale support with evaluator locales.
use super::{value_len, EvalError, EvaluatedArgument, RuntimeState, Sexp};
use r_ust_sexp::{is_na_real, ListElement};
use std::cmp::Ordering;

#[derive(Clone, PartialEq)]
enum Group {
    Number(f64),
    Text(String),
    Missing,
    Nan,
}

fn groups(value: &Sexp) -> Result<Vec<Group>, EvalError> {
    Ok(match value {
        Sexp::Integer(v) => v
            .iter()
            .map(|v| v.map_or(Group::Missing, |v| Group::Number(f64::from(v))))
            .collect(),
        Sexp::Logical(v) => v
            .iter()
            .map(|v| v.map_or(Group::Missing, |v| Group::Number(f64::from(u8::from(v)))))
            .collect(),
        Sexp::Real(v) => v
            .iter()
            .map(|v| {
                if is_na_real(*v) {
                    Group::Missing
                } else if v.is_nan() {
                    Group::Nan
                } else {
                    Group::Number(*v)
                }
            })
            .collect(),
        Sexp::String(v) | Sexp::AttributedCharacter { values: v, .. } => v
            .iter()
            .map(|v| v.clone().map_or(Group::Missing, Group::Text))
            .collect(),
        Sexp::Factor { codes, .. } => codes
            .iter()
            .map(|v| {
                v.map_or(Ok(Group::Missing), |v| {
                    u32::try_from(v)
                        .map(|v| Group::Number(f64::from(v)))
                        .map_err(|_| EvalError::Domain("factor code is too large".into()))
                })
            })
            .collect::<Result<_, _>>()?,
        Sexp::Matrix { data, .. } => return groups(data),
        Sexp::Nil => Vec::new(),
        _ => return Err(EvalError::NotImplemented("rowsum group type")),
    })
}

fn compare(a: &Group, b: &Group) -> Ordering {
    match (a, b) {
        (Group::Number(a), Group::Number(b)) => a.partial_cmp(b).unwrap_or(Ordering::Equal),
        (Group::Text(a), Group::Text(b)) => a.cmp(b),
        (Group::Missing | Group::Nan, Group::Missing | Group::Nan) => Ordering::Equal,
        (Group::Missing | Group::Nan, _) => Ordering::Greater,
        (_, Group::Missing | Group::Nan) => Ordering::Less,
        _ => Ordering::Equal,
    }
}

impl RuntimeState {
    pub(super) fn call_rowsum(
        &mut self,
        validate: bool,
        args: &[EvaluatedArgument],
    ) -> Result<Sexp, EvalError> {
        let expected = if validate { 2 } else { 4 };
        if args.len() != expected {
            return Err(EvalError::Arity("invalid rowsum helper arguments".into()));
        }
        let x = &args[0].value;
        let g = groups(&args[1].value)?;
        let rows = match x {
            Sexp::Matrix { nrow, .. } | Sexp::DataFrame { nrow, .. } => *nrow,
            _ => value_len(x)?,
        };
        if g.len() != rows {
            return Err(EvalError::Domain("incorrect length for 'group'".into()));
        }
        if validate {
            if g.iter().any(|g| matches!(g, Group::Missing | Group::Nan)) {
                self.record_parse_warnings(&["missing values for 'group'".into()])?;
            }
            return Ok(Sexp::Nil);
        }
        let reorder = super::internal::as_logical(&args[2].value)
            .ok_or_else(|| EvalError::Domain("invalid reorder".into()))?;
        let remove = super::internal::as_logical(&args[3].value)
            .ok_or_else(|| EvalError::Domain("'na.rm' must be TRUE or FALSE".into()))?;
        let mut unique = Vec::new();
        for key in &g {
            if !unique.contains(key) {
                unique.push(key.clone());
            }
        }
        if reorder {
            unique.sort_by(compare);
        }
        let indices = g
            .iter()
            .map(|key| {
                unique
                    .iter()
                    .position(|v| v == key)
                    .ok_or_else(|| EvalError::Domain("unmatched rowsum group".into()))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let grouping = Grouping {
            indices,
            count: unique.len(),
            remove,
        };
        match x {
            Sexp::DataFrame { columns, .. } => Ok(Sexp::DataFrame {
                nrow: grouping.count,
                columns: columns
                    .iter()
                    .map(|column| {
                        Ok(ListElement {
                            name: column.name.clone(),
                            value: grouping.sum(&column.value)?,
                        })
                    })
                    .collect::<Result<_, EvalError>>()?,
            }),
            Sexp::Matrix { data, ncol, .. } => Ok(Sexp::Matrix {
                data: Box::new(grouping.sum(data)?),
                nrow: grouping.count,
                ncol: *ncol,
            }),
            _ => Ok(Sexp::Matrix {
                data: Box::new(grouping.sum(x)?),
                nrow: grouping.count,
                ncol: 1,
            }),
        }
    }
}

struct Grouping {
    indices: Vec<usize>,
    count: usize,
    remove: bool,
}
impl Grouping {
    fn sum(&self, x: &Sexp) -> Result<Sexp, EvalError> {
        let rows = self.indices.len();
        let columns = value_len(x)?.checked_div(rows).unwrap_or(1);
        match x {
            Sexp::Integer(values) => {
                let mut out = vec![Some(0i32); self.count * columns];
                for (i, value) in values.iter().enumerate() {
                    let target = self.indices[i % rows] + (i / rows) * self.count;
                    match value {
                        None if !self.remove => out[target] = None,
                        Some(value) => {
                            out[target] = out[target]
                                .and_then(|sum| sum.checked_add(*value))
                                .filter(|sum| *sum != i32::MIN)
                        }
                        None => {}
                    }
                }
                Ok(Sexp::Integer(out))
            }
            Sexp::Real(values) => {
                let mut out = vec![0.0; self.count * columns];
                for (i, value) in values.iter().enumerate() {
                    if !self.remove || !value.is_nan() {
                        let target = self.indices[i % rows] + (i / rows) * self.count;
                        out[target] += value;
                    }
                }
                Ok(Sexp::Real(out))
            }
            _ => Err(EvalError::Type("non-numeric data frame in rowsum".into())),
        }
    }
}
