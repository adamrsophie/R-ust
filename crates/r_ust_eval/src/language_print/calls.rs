use super::{
    names,
    operators::{self, Kind},
    LanguagePrintError, Writer,
};
use crate::{CallArgument, ListElement, Sexp};
impl Writer {
    pub(super) fn call(
        &mut self,
        head: &Sexp,
        args: &[CallArgument],
        context: (bool, u8),
    ) -> Result<(), LanguagePrintError> {
        let (fnarg, left) = context;
        match head {
            Sexp::Symbol(name) => {
                names::check_text(name)?;
                for arg in args {
                    if let Some(tag) = &arg.name {
                        names::check_text(tag)?;
                    }
                }
                let mut op = operators::info(name);
                if operators::user_binop(name)
                    && !super::primitives::NAMES.contains(&name.as_str())
                    && args.iter().any(|arg| arg.name.is_some())
                {
                    op.kind = Kind::Call;
                }
                match op.kind {
                    Kind::Binary => match args.len() {
                        1 => {
                            op.kind = Kind::Unary;
                            if op.precedence == 9 {
                                op.precedence = 13;
                            }
                        }
                        2 => {}
                        _ => op.kind = Kind::Call,
                    },
                    Kind::Binary2 => {
                        if args.len() != 2 {
                            op.kind = Kind::Call;
                        } else if operators::user_binop(name)
                            && !super::primitives::NAMES.contains(&name.as_str())
                        {
                            op.kind = Kind::Binary;
                        }
                    }
                    Kind::Dollar => match args {
                        [_, rhs] if matches!(&rhs.value, Sexp::Symbol(_)) => {}
                        [_, rhs] if matches!(&rhs.value, Sexp::String(values) if matches!(values.as_slice(), [Some(_)])) =>
                            {}
                        _ => op.kind = Kind::Call,
                    },
                    _ => {}
                }
                match op.kind {
                    Kind::Call | Kind::Subassign => {
                        self.emit(&names::name(name)?);
                        self.emit("(");
                        let list = op.kind == Kind::Call
                            && (super::primitives::NAMES.contains(&name.as_str())
                                || operators::user_binop(name));
                        self.inlist += usize::from(list);
                        self.arguments(args)?;
                        self.inlist -= usize::from(list);
                        self.emit(")");
                        Ok(())
                    }
                    Kind::Binary | Kind::Binary2 | Kind::Unary | Kind::Assign | Kind::Dollar => {
                        self.infix((name, op), args, (fnarg, left))
                    }
                    Kind::Subset => {
                        let Some((target, indices)) = args.split_first() else {
                            return Err(LanguagePrintError::MalformedCall);
                        };
                        let parens = operators::needs_parens(op, &target.value, (true, left));
                        if parens {
                            self.emit("(");
                        }
                        self.write(&target.value)?;
                        if parens {
                            self.emit(")");
                        }
                        self.emit(name);
                        self.arguments(indices)?;
                        self.emit(if name == "[" { "]" } else { "]]" });
                        Ok(())
                    }
                    Kind::If
                    | Kind::While
                    | Kind::For
                    | Kind::Repeat
                    | Kind::Curly
                    | Kind::Paren
                    | Kind::Function
                    | Kind::Break
                    | Kind::Next => self.control(op.kind, args),
                }
            }
            Sexp::String(_) => Err(LanguagePrintError::MalformedCall),
            _ => {
                let parens = operators::parenthesize_caller(head);
                if parens {
                    self.emit("(");
                }
                self.write(head)?;
                if parens {
                    self.emit(")");
                }
                self.emit("(");
                self.arguments(args)?;
                self.emit(")");
                Ok(())
            }
        }
    }
    pub(super) fn arguments(&mut self, args: &[CallArgument]) -> Result<(), LanguagePrintError> {
        let mut continued = false;
        for (index, arg) in args.iter().enumerate() {
            if let Some(tag) = &arg.name {
                self.emit(&names::name(tag)?);
                self.emit(" = ");
            }
            self.fnarg = true;
            self.write(&arg.value)?;
            if index + 1 < args.len() {
                self.emit(", ");
                self.linebreak(&mut continued);
            }
        }
        if continued {
            self.indent -= 1;
        }
        Ok(())
    }
    pub(super) fn formals(&mut self, args: &[ListElement]) -> Result<(), LanguagePrintError> {
        let mut continued = false;
        for (index, arg) in args.iter().enumerate() {
            let Some(tag) = &arg.name else {
                return Err(LanguagePrintError::MalformedCall);
            };
            self.emit(&names::name(tag)?);
            if !matches!(arg.value, Sexp::MissingValue) {
                self.emit(" = ");
                self.fnarg = true;
                self.write(&arg.value)?;
            }
            if index + 1 < args.len() {
                self.emit(", ");
                self.linebreak(&mut continued);
            }
        }
        if continued {
            self.indent -= 1;
        }
        Ok(())
    }
}
