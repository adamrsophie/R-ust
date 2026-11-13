use super::{
    BinaryOp, CallArgument, EvaluationOrigin, FunctionParameter, ListElement, PromiseState, UnaryOp,
};

#[derive(Clone, Debug, PartialEq)]
pub enum ClosureEnvironment {
    Captured(Vec<(String, Sexp)>),
    Base,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InternalOperation {
    Vector,
    Stop,
    Inherits,
    Gettext,
    SignalCondition,
    DefaultStop,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Sexp {
    Nil,
    MissingArg(String),
    Symbol(String),
    Logical(Vec<Option<bool>>),
    Integer(Vec<Option<i32>>),
    Real(Vec<f64>),
    String(Vec<Option<String>>),
    Vector(Vec<Sexp>),
    List(Vec<ListElement>),
    Factor {
        codes: Vec<Option<usize>>,
        levels: Vec<String>,
        ordered: bool,
    },
    DataFrame {
        columns: Vec<ListElement>,
        nrow: usize,
    },
    Matrix {
        data: Box<Sexp>,
        nrow: usize,
        ncol: usize,
    },
    Dots(Vec<ListElement>),
    Promise {
        expression: Box<Sexp>,
        environment: Vec<(String, Sexp)>,
        origin: EvaluationOrigin,
        state: std::sync::Arc<PromiseState>,
        supplied: bool,
    },
    Closure {
        parameters: Vec<FunctionParameter>,
        body: Box<Sexp>,
        environment: ClosureEnvironment,
    },
    InternalCall {
        operation: InternalOperation,
        arguments: Vec<CallArgument>,
    },
    Function {
        parameters: Vec<FunctionParameter>,
        body: Box<Sexp>,
    },
    If {
        condition: Box<Sexp>,
        then_branch: Box<Sexp>,
        else_branch: Option<Box<Sexp>>,
    },
    For {
        variable: String,
        sequence: Box<Sexp>,
        body: Box<Sexp>,
    },
    While {
        condition: Box<Sexp>,
        body: Box<Sexp>,
    },
    Repeat {
        body: Box<Sexp>,
    },
    Break,
    Next,
    Block(Vec<Sexp>),
    Subset {
        value: Box<Sexp>,
        index: Option<Box<Sexp>>,
    },
    Subset2D {
        value: Box<Sexp>,
        row: Option<Box<Sexp>>,
        col: Option<Box<Sexp>>,
    },
    Extract {
        value: Box<Sexp>,
        index: Box<Sexp>,
    },
    Dollar {
        value: Box<Sexp>,
        name: String,
    },
    Call {
        function: Box<Sexp>,
        arguments: Vec<CallArgument>,
    },
    Assignment {
        target: String,
        value: Box<Sexp>,
    },
    SuperAssignment {
        target: String,
        value: Box<Sexp>,
    },
    Subassignment {
        target: String,
        index: Option<Box<Sexp>>,
        value: Box<Sexp>,
    },
    MatrixSubassignment {
        target: String,
        row: Option<Box<Sexp>>,
        col: Option<Box<Sexp>>,
        value: Box<Sexp>,
    },
    ExtractAssignment {
        target: String,
        index: Box<Sexp>,
        value: Box<Sexp>,
    },
    DollarAssignment {
        target: String,
        name: String,
        value: Box<Sexp>,
    },
    NamesAssignment {
        target: String,
        value: Box<Sexp>,
    },
    Binary {
        op: BinaryOp,
        left: Box<Sexp>,
        right: Box<Sexp>,
    },
    Unary {
        op: UnaryOp,
        value: Box<Sexp>,
    },
    Parenthesized(Box<Sexp>),
}
