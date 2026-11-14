use super::{
    names,
    operators::{needs_parens, Info, Kind},
    LanguagePrintError, Writer,
};
use crate::{CallArgument, Sexp};
impl Writer {
    pub(super) fn infix(
        &mut self,
        operator: (&str, Info),
        args: &[CallArgument],
        context: (bool, u8),
    ) -> Result<(), LanguagePrintError> {
        let (name, op) = operator;
        let (fnarg, previous) = context;
        let outer = op.kind == Kind::Assign && fnarg && name == "=";
        if outer {
            self.emit("(");
        }
        if op.kind == Kind::Unary {
            let [arg] = args else {
                return Err(LanguagePrintError::MalformedCall);
            };
            self.emit(name);
            self.operand(&arg.value, op, (false, previous))?;
        } else {
            let [lhs, rhs] = args else {
                return Err(LanguagePrintError::MalformedCall);
            };
            self.operand(&lhs.value, op, (true, previous))?;
            let spaces = matches!(op.kind, Kind::Binary | Kind::Assign);
            if spaces {
                self.emit(" ");
            }
            self.emit(name);
            if spaces {
                self.emit(" ");
            }
            let mut continued = false;
            if op.kind == Kind::Binary {
                self.linebreak(&mut continued);
            }
            if op.kind == Kind::Dollar {
                if let Sexp::String(values) = &rhs.value {
                    let [Some(member)] = values.as_slice() else {
                        return Err(LanguagePrintError::MalformedCall);
                    };
                    names::check_text(member)?;
                    if names::valid_name(member) {
                        self.emit(member);
                    } else {
                        self.operand(&rhs.value, op, (false, previous))?;
                    }
                } else {
                    self.operand(&rhs.value, op, (false, previous))?;
                }
            } else {
                self.operand(&rhs.value, op, (false, previous))?;
            }
            if continued {
                self.indent -= 1;
            }
        }
        if outer {
            self.emit(")");
        }
        self.left = 0;
        Ok(())
    }
    fn operand(
        &mut self,
        value: &Sexp,
        op: Info,
        side: (bool, u8),
    ) -> Result<(), LanguagePrintError> {
        let (left, deep) = side;
        let parens = needs_parens(op, value, (left, deep));
        if parens {
            self.emit("(");
        }
        self.left = if parens {
            0
        } else if left {
            op.precedence
        } else {
            deep
        };
        self.write(value)?;
        if parens {
            self.emit(")");
        }
        Ok(())
    }
}
