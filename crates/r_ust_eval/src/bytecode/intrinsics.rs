pub(super) fn print_flag(name: &str) -> Option<u8> {
    match name {
        "eval" | "do.call" => Some(2),
        "quit" | "print.default" | ".addCondHands" | ".resetCondHands" | "warning" | "assign" => {
            Some(1)
        }
        "commandArgs" | "readline" | "environment" | "new.env" | "parent.frame" | "sys.call"
        | "sys.frame" | "sys.function" | "sys.parent" | "sys.nframe" | "parent.env"
        | "environmentName" | "formals" | "body" | "bitwiseNot" | "bitwiseAnd" | "bitwiseOr"
        | "bitwiseXor" | "bitwiseShiftL" | "bitwiseShiftR" | "encodeString" | "sequence"
        | "utf8ToInt" | "intToUtf8" | "findInterval" | "strtoi" | "pmatch" | "charmatch"
        | "is.vector" | "as.vector" | "vector" | "stop" | "inherits" | "gettext"
        | ".signalCondition" | ".dfltStop" | "geterrmessage" | "get" | ".addRestart"
        | ".getRestart" | ".invokeRestart" | ".dfltWarn" | "identical" | "sprintf" | "deparse"
        | "ngettext" | "l10n_info" => Some(0),
        _ => None,
    }
}
