use super::{deparse_attributes::attribute, LanguagePrintError, Writer};
use crate::{EvalError, Sexp};

impl Writer<'_> {
    pub(super) fn deparse_special(&mut self, value: &Sexp) -> Result<(), LanguagePrintError> {
        match value {
            Sexp::Factor {
                codes,
                levels,
                ordered,
            } => {
                let codes = codes
                    .iter()
                    .map(|code| code.map(i32::try_from).transpose())
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|_| {
                        LanguagePrintError::from(EvalError::Domain("factor code overflow".into()))
                    })?;
                let class = if *ordered {
                    vec![Some("ordered".into()), Some("factor".into())]
                } else {
                    vec![Some("factor".into())]
                };
                let attributes = [
                    attribute(
                        "levels",
                        Sexp::String(levels.iter().cloned().map(Some).collect()),
                    ),
                    attribute("class", Sexp::String(class)),
                ];
                self.deparse_atomic(&Sexp::Integer(codes), &attributes)
            }
            Sexp::Matrix { data, nrow, ncol } => {
                let dimensions = [*nrow, *ncol]
                    .into_iter()
                    .map(|n| i32::try_from(n).map(Some))
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|_| {
                        LanguagePrintError::from(EvalError::Domain(
                            "matrix dimension overflow".into(),
                        ))
                    })?;
                let attributes = [attribute("dim", Sexp::Integer(dimensions))];
                if matches!(
                    data.as_ref(),
                    Sexp::Logical(_) | Sexp::Integer(_) | Sexp::Real(_) | Sexp::String(_)
                ) {
                    self.deparse_atomic(data, &attributes)
                } else {
                    let plan = self.begin_attributes(&attributes, false, false);
                    self.write(data)?;
                    self.end_attributes(plan)
                }
            }
            Sexp::DataFrame { columns, nrow } => {
                let nrow = i32::try_from(*nrow).map_err(|_| {
                    LanguagePrintError::from(EvalError::Domain(
                        "data frame row count overflow".into(),
                    ))
                })?;
                let rows = if nrow == 0 {
                    Vec::new()
                } else {
                    vec![None, Some(-nrow)]
                };
                let attributes = [
                    attribute(
                        "names",
                        Sexp::String(columns.iter().map(|column| column.name.clone()).collect()),
                    ),
                    attribute("class", Sexp::String(vec![Some("data.frame".into())])),
                    attribute("row.names", Sexp::Integer(rows)),
                ];
                self.deparse_list(columns, &attributes, false)
            }
            _ => Err(LanguagePrintError::UnsupportedNode),
        }
    }
}
