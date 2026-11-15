use super::{operators::Kind, LanguagePrintError, Writer};
use crate::{CallArgument, Sexp};
fn curly(value: &Sexp) -> bool {
    matches!(value, Sexp::Call { function, .. } if matches!(function.as_ref(), Sexp::Symbol(name) if name == "{"))
}
impl Writer<'_> {
    pub(super) fn control(
        &mut self,
        kind: Kind,
        args: &[CallArgument],
    ) -> Result<(), LanguagePrintError> {
        if self.profile.is_none() {
            return self.control_shape(kind, args);
        }
        if kind == Kind::Function && self.options() & super::profile::SOURCE != 0 {
            if args
                .get(2)
                .is_some_and(|source| crate::is_character_value(&source.value))
            {
                return Err(
                    crate::EvalError::NotImplemented("deparse function source text").into(),
                );
            }
        }
        let count = match kind {
            Kind::If => {
                if args.len() > 2 {
                    3
                } else {
                    2
                }
            }
            Kind::While => 2,
            Kind::For | Kind::Function => 3,
            Kind::Paren | Kind::Repeat => 1,
            Kind::Break | Kind::Next => 0,
            _ => return self.control_shape(kind, args),
        };
        let positional = (0..count)
            .map(|index| CallArgument {
                name: None,
                value: if kind == Kind::Function && index == 2 {
                    Sexp::Nil
                } else {
                    args.get(index).map_or(Sexp::Nil, |arg| arg.value.clone())
                },
            })
            .collect::<Vec<_>>();
        self.control_shape(kind, &positional)
    }
    fn control_shape(
        &mut self,
        kind: Kind,
        args: &[CallArgument],
    ) -> Result<(), LanguagePrintError> {
        match (kind, args) {
            (Kind::Curly, _) => {
                self.emit("{");
                self.incurly += 1;
                self.indent += 1;
                self.newline();
                for arg in args {
                    self.write(&arg.value)?;
                    self.newline();
                }
                self.indent -= 1;
                self.emit("}");
                self.incurly -= 1;
            }
            (Kind::Paren, [arg]) => {
                self.emit("(");
                self.write(&arg.value)?;
                self.emit(")");
            }
            (Kind::Break, []) => self.emit("break"),
            (Kind::Next, []) => self.emit("next"),
            (Kind::Repeat, [body]) => {
                self.emit("repeat ");
                self.write(&body.value)?;
            }
            (Kind::While, [condition, body]) => {
                self.emit("while (");
                self.write(&condition.value)?;
                self.emit(") ");
                self.write(&body.value)?;
            }
            (Kind::For, [variable, sequence, body])
                if self.profile.is_some() || matches!(variable.value, Sexp::Symbol(_)) =>
            {
                self.emit("for (");
                self.write(&variable.value)?;
                self.emit(" in ");
                self.write(&sequence.value)?;
                self.emit(") ");
                self.write(&body.value)?;
            }
            (Kind::If, [condition, yes]) => {
                self.if_expression(&condition.value, &yes.value, None)?
            }
            (Kind::If, [condition, yes, no]) => {
                self.if_expression(&condition.value, &yes.value, Some(&no.value))?
            }
            (Kind::Function, [formals, body, source]) if matches!(source.value, Sexp::Nil) => {
                self.emit("function(");
                let formals = if self.profile.is_some() {
                    crate::ordinary_values::read(&formals.value)?
                } else {
                    std::borrow::Cow::Borrowed(&formals.value)
                };
                match formals.as_ref() {
                    Sexp::Nil => {}
                    Sexp::PairList(values) => self.formals(values.as_slice())?,
                    _ => return Err(LanguagePrintError::MalformedCall),
                }
                self.emit(") ");
                self.write(&body.value)?;
            }
            _ => return Err(LanguagePrintError::MalformedCall),
        }
        Ok(())
    }
    fn if_expression(
        &mut self,
        condition: &Sexp,
        yes: &Sexp,
        no: Option<&Sexp>,
    ) -> Result<(), LanguagePrintError> {
        self.emit("if (");
        self.write(condition)?;
        self.emit(") ");
        let nested = self.incurly > 0 && self.inlist == 0;
        let lookahead = nested && curly(yes);
        if nested && !lookahead {
            self.newline();
            self.indent += 1;
        }
        self.write(yes)?;
        if let Some(no) = no {
            if nested {
                self.newline();
                if !lookahead {
                    self.indent -= 1;
                }
            } else {
                self.emit(" ");
            }
            self.emit("else ");
            self.write(no)?;
        } else if nested && !lookahead {
            self.indent -= 1;
        }
        Ok(())
    }
}
