use crate::Sexp;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Kind {
    Call,
    Binary,
    Binary2,
    Unary,
    Assign,
    Dollar,
    Subset,
    Subassign,
    If,
    While,
    For,
    Repeat,
    Curly,
    Paren,
    Function,
    Break,
    Next,
}
#[derive(Clone, Copy)]
pub(super) struct Info {
    pub(super) kind: Kind,
    pub(super) precedence: u8,
    pub(super) right: bool,
}
pub(super) fn user_binop(name: &str) -> bool {
    name.len() >= 2 && name.starts_with('%') && name.ends_with('%')
}
pub(super) fn info(name: &str) -> Info {
    use Kind::*;
    let (kind, precedence, right) = match name {
        "if" => (If, 0, true),
        "while" => (While, 0, false),
        "for" => (For, 0, false),
        "repeat" => (Repeat, 0, false),
        "{" => (Curly, 0, false),
        "(" => (Paren, 0, false),
        "function" => (Function, 0, false),
        "break" => (Break, 0, false),
        "next" => (Next, 0, false),
        "=" => (Assign, 1, true),
        "<-" | "<<-" => (Assign, 2, true),
        "~" => (Binary, 4, false),
        "|" | "||" => (Binary, 5, false),
        "&" | "&&" => (Binary, 6, false),
        "!" => (Unary, 7, false),
        "==" | "!=" | "<" | "<=" | ">" | ">=" => (Binary, 8, false),
        "+" | "-" => (Binary, 9, false),
        "*" => (Binary, 10, false),
        "/" => (Binary2, 10, false),
        "%*%" => (Binary, 11, false),
        "%%" | "%/%" => (Binary2, 11, false),
        ":" => (Binary2, 12, false),
        "^" => (Binary2, 14, true),
        "[" | "[[" => (Subset, 15, false),
        "$" | "@" => (Dollar, 16, false),
        "::" | ":::" => (Binary2, 17, false),
        "[<-" | "[[<-" | "$<-" | "@<-" => (Subassign, 2, true),
        "length<-" | "oldClass<-" | "class<-" | "names<-" | "dimnames<-" | "dim<-"
        | "attributes<-" | "attr<-" | "levels<-" => (Call, 2, true),
        name if user_binop(name) => (Binary2, 11, false),
        _ => (Call, 0, false),
    };
    Info {
        kind,
        precedence,
        right,
    }
}
pub(super) fn needs_parens(main: Info, arg: &Sexp, side: (bool, u8)) -> bool {
    let (left, deep) = side;
    let Sexp::Call {
        function,
        arguments,
    } = arg
    else {
        return false;
    };
    let Sexp::Symbol(name) = function.as_ref() else {
        return false;
    };
    if user_binop(name) && !super::primitives::NAMES.contains(&name.as_str()) {
        return main.precedence > 11 || (main.precedence == 11 && left == main.right);
    }
    let mut arg = info(name);
    if matches!(arg.kind, Kind::Binary | Kind::Binary2) {
        match arguments.len() {
            1 => {
                if arg.precedence == 9 {
                    arg.precedence = 13;
                }
                arg.kind = Kind::Unary;
            }
            2 => {}
            _ => return false,
        }
    }
    match arg.kind {
        Kind::Subset
            if matches!(main.kind, Kind::Dollar | Kind::Subset)
                && main.precedence > arg.precedence =>
        {
            false
        }
        Kind::Binary | Kind::Binary2 if main.precedence == 8 && arg.precedence == 8 => true,
        Kind::Subset | Kind::Binary | Kind::Binary2 | Kind::Assign | Kind::Dollar => {
            main.precedence > arg.precedence
                || (main.precedence == arg.precedence && left == main.right)
        }
        Kind::Unary => {
            (left && main.precedence > arg.precedence) || (deep > 0 && deep > arg.precedence)
        }
        Kind::For | Kind::If | Kind::While | Kind::Repeat => left || deep > 0,
        Kind::Call
        | Kind::Subassign
        | Kind::Curly
        | Kind::Paren
        | Kind::Function
        | Kind::Break
        | Kind::Next => false,
    }
}
pub(super) fn parenthesize_caller(head: &Sexp) -> bool {
    match head {
        Sexp::Call { function, .. } => match function.as_ref() {
            Sexp::Symbol(name) => {
                if user_binop(name)
                    || matches!(
                        name.as_str(),
                        "return"
                            | ".C"
                            | ".Fortran"
                            | ".External"
                            | ".External2"
                            | ".Call"
                            | ".External.graphics"
                            | ".Call.graphics"
                    )
                {
                    return true;
                }
                let info = info(name);
                info.precedence < 15 && !matches!(info.kind, Kind::Call | Kind::Paren | Kind::Curly)
            }
            _ => true,
        },
        _ => false,
    }
}
