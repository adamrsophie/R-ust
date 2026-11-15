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

    if let Some(view) = value.ordinary_list() {
        return Ok(SamplePopulation {
            values: view.elements.to_vec(),
            output: SampleOutputKind::List {
                names_present: view.names_present,
            },
        });
    }
    match value {
        Sexp::ClassedList {
            elements: values, ..
        }
        | Sexp::List(values)
        | Sexp::Dots(values) => Ok(SamplePopulation {
            values: values.clone(),
            output: SampleOutputKind::List {
                names_present: values.iter().any(|element| element.name.is_some()),
            },
        }),
        Sexp::DataFrame { columns, .. } => Ok(SamplePopulation {
            values: columns.clone(),
            output: SampleOutputKind::List {
                names_present: columns.iter().any(|element| element.name.is_some()),
            },
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
