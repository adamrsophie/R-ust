use super::{operators::Kind, LanguagePrintError, Writer};
use crate::{CallArgument, Sexp};
fn curly(value: &Sexp) -> bool {
    matches!(value, Sexp::Call { function, .. } if matches!(function.as_ref(), Sexp::Symbol(name) if name == "{"))
}
impl Writer {
    pub(super) fn control(
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
                if matches!(variable.value, Sexp::Symbol(_)) =>
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
                match &formals.value {
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
