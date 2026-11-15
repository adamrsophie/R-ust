use super::{
    profile::{SIMPLE, SOURCE},
    LanguagePrintError, Writer,
};
use crate::{EvalError, ListElement, Sexp};

impl Writer<'_> {
    pub(super) fn deparse_closure(&mut self, value: &Sexp) -> Result<(), LanguagePrintError> {
        let Sexp::Closure {
            parameters, body, ..
        } = value
        else {
            return Err(LanguagePrintError::UnsupportedNode);
        };
        let attributes =
            crate::closure_attributes::attributes(value).map_err(LanguagePrintError::from)?;
        if self.options() & SOURCE != 0
            && attributes
                .iter()
                .any(|a| a.name.as_deref() == Some("srcref") && !matches!(a.value, Sexp::Nil))
        {
            return Err(EvalError::NotImplemented("deparse source-reference provenance").into());
        }
        let options = self.options();
        let plan = self.begin_attributes(attributes, false, false);
        self.set_options(options & SIMPLE & !SOURCE);
        let formals = parameters
            .iter()
            .map(|parameter| ListElement {
                name: Some(parameter.name.clone()),
                value: parameter.default.clone().unwrap_or(Sexp::MissingValue),
            })
            .collect::<Vec<_>>();
        self.emit("function (");
        self.formals(&formals)?;
        self.emit(") ");
        self.newline();
        let result = self.write(body);
        self.set_options(options);
        result?;
        self.end_attributes(plan)
    }
}
