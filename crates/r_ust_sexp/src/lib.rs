#![allow(non_camel_case_types)]

pub const R_VERSION: &str = "4.6.0";

// R's arithmetic.c identifies NA by the low payload word, even after NaN quieting.
pub const NA_REAL: f64 = f64::from_bits(0x7ff0_0000_0000_07a2);

pub fn is_na_real(value: f64) -> bool {
    value.is_nan() && value.to_bits() as u32 == 1954
}

pub type R_xlen_t = isize;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum SexpType {
    Nil = 0,
    Symbol = 1,
    List = 2,
    Closure = 3,
    Environment = 4,
    Promise = 5,
    Language = 6,
    Special = 7,
    Builtin = 8,
    Char = 9,
    Logical = 10,
    Integer = 13,
    Real = 14,
    Complex = 15,
    String = 16,
    Dots = 17,
    Any = 18,
    Vector = 19,
    Expression = 20,
    Bytecode = 21,
    ExternalPtr = 22,
    WeakRef = 23,
    Raw = 24,
    Object = 25,
    New = 30,
    Free = 31,
    Function = 99,
}

impl SexpType {
    pub const fn from_r_type(value: u32) -> Option<Self> {
        match value {
            0 => Some(Self::Nil),
            1 => Some(Self::Symbol),
            2 => Some(Self::List),
            3 => Some(Self::Closure),
            4 => Some(Self::Environment),
            5 => Some(Self::Promise),
            6 => Some(Self::Language),
            7 => Some(Self::Special),
            8 => Some(Self::Builtin),
            9 => Some(Self::Char),
            10 => Some(Self::Logical),
            13 => Some(Self::Integer),
            14 => Some(Self::Real),
            15 => Some(Self::Complex),
            16 => Some(Self::String),
            17 => Some(Self::Dots),
            18 => Some(Self::Any),
            19 => Some(Self::Vector),
            20 => Some(Self::Expression),
            21 => Some(Self::Bytecode),
            22 => Some(Self::ExternalPtr),
            23 => Some(Self::WeakRef),
            24 => Some(Self::Raw),
            25 => Some(Self::Object),
            30 => Some(Self::New),
            31 => Some(Self::Free),
            99 => Some(Self::Function),
            _ => None,
        }
    }

    pub const fn as_r_type(self) -> u32 {
        self as u32
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SexpHeader {
    sexptype: SexpType,
    named: u8,
    object: bool,
    altrep: bool,
    gp: u16,
}

impl SexpHeader {
    pub const fn new(sexptype: SexpType) -> Self {
        Self {
            sexptype,
            named: 0,
            object: false,
            altrep: false,
            gp: 0,
        }
    }

    pub const fn sexptype(&self) -> SexpType {
        self.sexptype
    }

    pub const fn named(&self) -> u8 {
        self.named
    }

    pub const fn is_object(&self) -> bool {
        self.object
    }

    pub const fn is_altrep(&self) -> bool {
        self.altrep
    }

    pub const fn gp(&self) -> u16 {
        self.gp
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Power,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Equal,
    NotEqual,
    And,
    Or,
    AndScalar,
    OrScalar,
    In,
    Sequence,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnaryOp {
    Plus,
    Minus,
    Not,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CallArgument {
    pub name: Option<String>,
    pub value: Sexp,
}

impl CallArgument {
    pub fn positional(value: Sexp) -> Self {
        Self { name: None, value }
    }

    pub fn named(name: impl Into<String>, value: Sexp) -> Self {
        Self {
            name: Some(name.into()),
            value,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionParameter {
    pub name: String,
    pub default: Option<Sexp>,
}

impl FunctionParameter {
    pub fn required(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            default: None,
        }
    }

    pub fn optional(name: impl Into<String>, default: Sexp) -> Self {
        Self {
            name: name.into(),
            default: Some(default),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ListElement {
    pub name: Option<String>,
    pub value: Sexp,
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
        value: Option<Box<Sexp>>,
        supplied: bool,
    },
    Closure {
        parameters: Vec<FunctionParameter>,
        body: Box<Sexp>,
        environment: Vec<(String, Sexp)>,
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

impl Sexp {
    pub const fn sexptype(&self) -> SexpType {
        match self {
            Self::Nil => SexpType::Nil,
            Self::MissingArg(_) => SexpType::Promise,
            Self::Symbol(_) => SexpType::Symbol,
            Self::Logical(_) => SexpType::Logical,
            Self::Integer(_) => SexpType::Integer,
            Self::Real(_) => SexpType::Real,
            Self::String(_) => SexpType::String,
            Self::Vector(_) => SexpType::Vector,
            Self::List(_) => SexpType::List,
            Self::Factor { .. } => SexpType::Integer,
            Self::DataFrame { .. } => SexpType::List,
            Self::Matrix { .. } => SexpType::Vector,
            Self::Dots(_) => SexpType::Dots,
            Self::Promise { .. } => SexpType::Promise,
            Self::Closure { .. } => SexpType::Closure,
            Self::Function { .. }
            | Self::If { .. }
            | Self::For { .. }
            | Self::While { .. }
            | Self::Repeat { .. }
            | Self::Break
            | Self::Next
            | Self::Block(_)
            | Self::Subset { .. }
            | Self::Subset2D { .. }
            | Self::Extract { .. }
            | Self::Dollar { .. }
            | Self::Call { .. }
            | Self::Assignment { .. }
            | Self::SuperAssignment { .. }
            | Self::Subassignment { .. }
            | Self::MatrixSubassignment { .. }
            | Self::ExtractAssignment { .. }
            | Self::DollarAssignment { .. }
            | Self::NamesAssignment { .. }
            | Self::Binary { .. }
            | Self::Unary { .. } => SexpType::Language,
            Self::Parenthesized(_) => SexpType::Language,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{is_na_real, SexpType, NA_REAL};

    #[test]
    fn missing_double_payload_matches_r() {
        assert!(NA_REAL.is_nan());
        assert!(is_na_real(NA_REAL));
        assert!(is_na_real(-NA_REAL));
        assert!(is_na_real(NA_REAL + 1.0));
        assert!(!is_na_real(f64::NAN));
        assert!(!is_na_real(1954.0));
        assert!(!is_na_real(f64::INFINITY));
    }

    #[test]
    fn sexptype_values_match_r_4_6_0() {
        assert_eq!(SexpType::Nil.as_r_type(), 0);
        assert_eq!(SexpType::Closure.as_r_type(), 3);
        assert_eq!(SexpType::Integer.as_r_type(), 13);
        assert_eq!(SexpType::Raw.as_r_type(), 24);
        assert_eq!(SexpType::Object.as_r_type(), 25);
        assert_eq!(SexpType::Free.as_r_type(), 31);
        assert_eq!(SexpType::Function.as_r_type(), 99);
    }
}
