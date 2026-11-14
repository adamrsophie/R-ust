pub const PORT_TARGET: &str = "R 4.6.0 Unix CLI, implemented in Rust";

/// Display formatting returns a Result; callers must handle language and nested
/// container errors before emitting the completed text.
pub use r_ust_eval::format_sexp;
pub use r_ust_eval::{EvalError, Frontend, OutputEvent, QuitRequest, RuntimeState, SaveAction};
pub use r_ust_parse::{ParseError, ParseIter, ParseResult, ParseStatus, Parser};
pub use r_ust_sexp::{
    BinaryOp, CallArgument, FunctionParameter, Sexp, SexpHeader, SexpType, UnaryOp,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Component {
    pub crate_name: &'static str,
    pub purpose: &'static str,
}

pub const COMPONENTS: &[Component] = &[
    Component {
        crate_name: "r_ust_runtime",
        purpose: "Runtime composition and build metadata",
    },
    Component {
        crate_name: "r_ust_nmath",
        purpose: "Rmath/nmath algorithms and explicit RNG state",
    },
    Component {
        crate_name: "r_ust_sexp",
        purpose: "R object type model",
    },
    Component {
        crate_name: "r_ust_gc",
        purpose: "GC ownership and protect stack",
    },
    Component {
        crate_name: "r_ust_parse",
        purpose: "R parser entry points",
    },
    Component {
        crate_name: "r_ust_eval",
        purpose: "R evaluator and primitive registry",
    },
    Component {
        crate_name: "r_ust_ffi",
        purpose: "Rust-owned C ABI exports",
    },
    Component {
        crate_name: "r_ust_cli",
        purpose: "Unix command-line interface",
    },
];

pub fn component_names() -> impl Iterator<Item = &'static str> {
    COMPONENTS.iter().map(|component| component.crate_name)
}
