use super::*;

pub(super) fn bind_data_frame_columns(args: &[&EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let nrow = data_frame_cbind_nrow(args)?;
    let mut columns = Vec::new();

    for arg in args {
        let projection = ordinary_values::read(&arg.value)?;
        match projection.as_ref() {
            Sexp::Nil => {}
            Sexp::DataFrame {
                columns: source_columns,
                ..
            } => {
                for column in source_columns {
                    columns.push(ListElement {
                        name: column.name.clone(),
                        value: recycle_data_frame_column(&column.value, nrow)?,
                    });
                }
            }
            Sexp::Matrix {
                data,
                nrow: matrix_nrow,
                ncol,
            } => {
                for col in 0..*ncol {
                    let values = matrix_column_values(data, *matrix_nrow, col)?;
                    let value = scalar_elements_to_vector(&values)?;
                    let name = bind_column_name(arg.name.as_deref(), *ncol, col, columns.len());
                    columns.push(ListElement {
                        name: Some(name),
                        value: recycle_data_frame_column(&value, nrow)?,
                    });
                }
            }
            Sexp::ClassedList {
                elements: values, ..
            }
            | Sexp::List(values)
            | Sexp::Dots(values) => {
                for (index, element) in values.iter().enumerate() {
                    let name = element.name.clone().unwrap_or_else(|| {
                        bind_column_name(arg.name.as_deref(), values.len(), index, columns.len())
                    });
                    columns.push(ListElement {
                        name: Some(name),
                        value: recycle_data_frame_column(&element.value, nrow)?,
                    });
                }
            }
            value => {
                let name = arg
                    .name
                    .clone()
                    .unwrap_or_else(|| default_bind_column_name(columns.len()));
                columns.push(ListElement {
                    name: Some(name),
                    value: recycle_data_frame_column(value, nrow)?,
                });
            }
        }
    }

    Ok(Sexp::DataFrame { columns, nrow })
}

pub(super) fn bind_data_frame_rows(args: &[&EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let Some(first_arg) = args.first() else {
        return Ok(Sexp::Nil);
    };
    let Sexp::DataFrame {
        columns: template, ..
    } = &first_arg.value
    else {
        return Err(EvalError::Type(
            "rbind() with data frames expects the first argument to be a data frame".to_string(),
        ));
    };

    let ncol = template.len();
    let names = template
        .iter()
        .map(|column| column.name.clone())
        .collect::<Vec<_>>();
    let mut out_columns = vec![Vec::new(); ncol];
    let mut nrow = 0usize;

    for arg in args {
        let projection = ordinary_values::read(&arg.value)?;
        match projection.as_ref() {
            Sexp::Nil => {}
            Sexp::DataFrame {
                columns,
                nrow: rows,
            } => {
                let order = data_frame_rbind_column_order(template, columns)?;
                for (target_index, source_index) in order.into_iter().enumerate() {
                    let column = columns.get(source_index).ok_or_else(|| {
                        EvalError::Domain("data frame column index out of bounds".to_string())
                    })?;
                    let value = recycle_data_frame_column(&column.value, *rows)?;
                    out_columns[target_index].extend(sequence_elements(&value)?);
                }
                nrow += *rows;
            }
            Sexp::Matrix {
                data,
                nrow: matrix_nrow,
                ncol: matrix_ncol,
            } => {
                if *matrix_ncol != ncol {
                    return Err(EvalError::Arity(format!(
                        "rbind data-frame row has {matrix_ncol} values, expected {ncol}"
                    )));
                }
                for row in 0..*matrix_nrow {
                    append_data_frame_row(
                        &mut out_columns,
                        &matrix_row_values(data, (*matrix_nrow, *matrix_ncol), row)?,
                    )?;
                    nrow += 1;
                }
            }
            Sexp::ClassedList {
                elements: values, ..
            }
            | Sexp::List(values)
            | Sexp::Dots(values) => {
                let row = values
                    .iter()
                    .map(|element| element.value.clone())
                    .collect::<Vec<_>>();
                append_data_frame_row(&mut out_columns, &row)?;
                nrow += 1;
            }
            value => {
                append_data_frame_row(
                    &mut out_columns,
                    &recycle_bind_values(value, ncol, "rbind data-frame row")?,
                )?;
                nrow += 1;
            }
        }
    }

    let columns = names
        .into_iter()
        .zip(out_columns)
        .map(|(name, values)| {
            Ok(ListElement {
                name,
                value: scalar_elements_to_vector(&values)?,
            })
        })
        .collect::<Result<Vec<_>, EvalError>>()?;
    Ok(Sexp::DataFrame { columns, nrow })
}

pub(super) fn data_frame_cbind_nrow(args: &[&EvaluatedArgument]) -> Result<usize, EvalError> {
    let mut nrow = 0usize;
    for arg in args {
        let projection = ordinary_values::read(&arg.value)?;
        match projection.as_ref() {
            Sexp::Nil => {}
            Sexp::DataFrame { nrow: rows, .. } | Sexp::Matrix { nrow: rows, .. } => {
                nrow = nrow.max(*rows);
            }
            Sexp::ClassedList {
                elements: values, ..
            }
            | Sexp::List(values)
            | Sexp::Dots(values) => {
                for element in values {
                    nrow = nrow.max(value_len(&element.value)?);
                }
            }
            value => {
                nrow = nrow.max(value_len(value)?);
            }
        }
    }
    Ok(nrow)
}
