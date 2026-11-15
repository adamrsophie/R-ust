use r_ust_sexp::PrimitiveId;

pub(crate) fn format_primitive(id: PrimitiveId) -> String {
    let signature = match id {
        PrimitiveId::Syntax(id) => syntax_signature(id),
        PrimitiveId::Sum => "function (..., na.rm = FALSE)  ",
        PrimitiveId::List => "function (...)  ",
        PrimitiveId::AsCharacter => "function (x, ...)  ",
        PrimitiveId::Length | PrimitiveId::Missing | PrimitiveId::IsEnvironment => "function (x)  ",
        PrimitiveId::AsCall | PrimitiveId::AsEnvironment | PrimitiveId::IsSymbol => {
            "function (x)  "
        }
        PrimitiveId::NamespaceExport | PrimitiveId::NamespaceInternal => "function (pkg, name)  ",
        PrimitiveId::GlobalEnv
        | PrimitiveId::BaseEnv
        | PrimitiveId::EmptyEnv
        | PrimitiveId::Nargs => "function ()  ",
        PrimitiveId::Quote => "function (expr)  ",
        PrimitiveId::Expression => "function (...)  ",
        PrimitiveId::Call => "function (name, ...)  ",
        PrimitiveId::IsPairList => "function (x)  ",
        PrimitiveId::IsExpression => "function (x)  ",
        PrimitiveId::Substitute => "function (expr, env)  ",
        PrimitiveId::Primitive => "function (name)  ",
        PrimitiveId::Internal => "function (call)  ",
        PrimitiveId::Invisible => "function (x = NULL)  ",
        PrimitiveId::Return | PrimitiveId::Colon | PrimitiveId::InternalEval => "",
        PrimitiveId::UseMethod => "function (generic, object)  ",
        PrimitiveId::OnExit => "function (expr = NULL, add = FALSE, after = TRUE)  ",
        PrimitiveId::Add
        | PrimitiveId::Subtract
        | PrimitiveId::Multiply
        | PrimitiveId::Divide
        | PrimitiveId::Power
        | PrimitiveId::Modulo
        | PrimitiveId::IntegerDivide => "function (e1, e2)  ",
    };
    format!("{signature}.Primitive(\"{}\")", id.name())
}

fn syntax_signature(id: r_ust_sexp::SyntaxPrimitive) -> &'static str {
    use r_ust_sexp::SyntaxPrimitive as P;
    match id {
        P::And
        | P::Or
        | P::Equal
        | P::NotEqual
        | P::Less
        | P::LessEqual
        | P::Greater
        | P::GreaterEqual => "function (e1, e2)  ",
        P::Not => "function (x)  ",
        P::NamesAssign | P::ClassAssign | P::AttributesAssign | P::StorageModeAssign => {
            "function (x, value)  "
        }
        P::AttributeAssign => "function (x, which, value)  ",
        _ => "",
    }
}
