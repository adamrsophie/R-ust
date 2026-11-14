use r_ust_sexp::PrimitiveId;

pub(crate) fn format_primitive(id: PrimitiveId) -> String {
    let signature = match id {
        PrimitiveId::Sum => "function (..., na.rm = FALSE)  ",
        PrimitiveId::Length | PrimitiveId::Missing | PrimitiveId::IsEnvironment => "function (x)  ",
        PrimitiveId::GlobalEnv
        | PrimitiveId::BaseEnv
        | PrimitiveId::EmptyEnv
        | PrimitiveId::Nargs => "function ()  ",
        PrimitiveId::Quote => "function (expr)  ",
        PrimitiveId::Primitive => "function (name)  ",
        PrimitiveId::Invisible => "function (x = NULL)  ",
        PrimitiveId::Return | PrimitiveId::Colon => "",
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
