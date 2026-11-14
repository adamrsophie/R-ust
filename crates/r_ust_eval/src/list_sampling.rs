use super::*;

pub(super) fn sample_population(value: &Sexp) -> Result<SamplePopulation, EvalError> {
    let projection = crate::ordinary_values::read(value)?;
    let value = projection.as_ref();
    if let Some(n) = sample_integer_shortcut(value)? {
        return Ok(SamplePopulation {
            values: (1..=n)
                .map(|value| ListElement {
                    name: None,
                    value: Sexp::Integer(vec![Some(value as i32)]),
                })
                .collect(),
            output: SampleOutputKind::Atomic,
        });
    }

    match value {
        Sexp::ClassedList {
            elements: values, ..
        }
        | Sexp::List(values)
        | Sexp::Dots(values) => Ok(SamplePopulation {
            values: values.clone(),
            output: SampleOutputKind::List,
        }),
        Sexp::DataFrame { columns, .. } => Ok(SamplePopulation {
            values: columns.clone(),
            output: SampleOutputKind::List,
        }),
        other => Ok(SamplePopulation {
            values: sequence_elements(other)?
                .into_iter()
                .map(|value| ListElement { name: None, value })
                .collect(),
            output: SampleOutputKind::Atomic,
        }),
    }
}
