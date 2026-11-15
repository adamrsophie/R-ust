use crate::SexpType;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SyntaxPrimitive {
    If,
    While,
    For,
    Repeat,
    Break,
    Next,
    Function,
    Assign,
    EqualAssign,
    SuperAssign,
    Block,
    Parenthesis,
    Subset,
    Extract,
    Dollar,
    Subassign,
    ExtractAssign,
    DollarAssign,
    NamesAssign,
    ClassAssign,
    AttributesAssign,
    AttributeAssign,
    StorageModeAssign,
    AndScalar,
    OrScalar,
    And,
    Or,
    Not,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}

impl SyntaxPrimitive {
    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "if" => Self::If,
            "while" => Self::While,
            "for" => Self::For,
            "repeat" => Self::Repeat,
            "break" => Self::Break,
            "next" => Self::Next,
            "function" => Self::Function,
            "<-" => Self::Assign,
            "=" => Self::EqualAssign,
            "<<-" => Self::SuperAssign,
            "{" => Self::Block,
            "(" => Self::Parenthesis,
            "[" => Self::Subset,
            "[[" => Self::Extract,
            "$" => Self::Dollar,
            "[<-" => Self::Subassign,
            "[[<-" => Self::ExtractAssign,
            "$<-" => Self::DollarAssign,
            "names<-" => Self::NamesAssign,
            "class<-" => Self::ClassAssign,
            "attributes<-" => Self::AttributesAssign,
            "attr<-" => Self::AttributeAssign,
            "storage.mode<-" => Self::StorageModeAssign,
            "&&" => Self::AndScalar,
            "||" => Self::OrScalar,
            "&" => Self::And,
            "|" => Self::Or,
            "!" => Self::Not,
            "==" => Self::Equal,
            "!=" => Self::NotEqual,
            "<" => Self::Less,
            "<=" => Self::LessEqual,
            ">" => Self::Greater,
            ">=" => Self::GreaterEqual,
            _ => return None,
        })
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::If => "if",
            Self::While => "while",
            Self::For => "for",
            Self::Repeat => "repeat",
            Self::Break => "break",
            Self::Next => "next",
            Self::Function => "function",
            Self::Assign => "<-",
            Self::EqualAssign => "=",
            Self::SuperAssign => "<<-",
            Self::Block => "{",
            Self::Parenthesis => "(",
            Self::Subset => "[",
            Self::Extract => "[[",
            Self::Dollar => "$",
            Self::Subassign => "[<-",
            Self::ExtractAssign => "[[<-",
            Self::DollarAssign => "$<-",
            Self::NamesAssign => "names<-",
            Self::ClassAssign => "class<-",
            Self::AttributesAssign => "attributes<-",
            Self::AttributeAssign => "attr<-",
            Self::StorageModeAssign => "storage.mode<-",
            Self::AndScalar => "&&",
            Self::OrScalar => "||",
            Self::And => "&",
            Self::Or => "|",
            Self::Not => "!",
            Self::Equal => "==",
            Self::NotEqual => "!=",
            Self::Less => "<",
            Self::LessEqual => "<=",
            Self::Greater => ">",
            Self::GreaterEqual => ">=",
        }
    }

    pub const fn sexptype(self) -> SexpType {
        match self {
            Self::Parenthesis
            | Self::NamesAssign
            | Self::ClassAssign
            | Self::AttributesAssign
            | Self::AttributeAssign
            | Self::StorageModeAssign
            | Self::And
            | Self::Or
            | Self::Not
            | Self::Equal
            | Self::NotEqual
            | Self::Less
            | Self::LessEqual
            | Self::Greater
            | Self::GreaterEqual => SexpType::Builtin,
            Self::If
            | Self::While
            | Self::For
            | Self::Repeat
            | Self::Break
            | Self::Next
            | Self::Function
            | Self::Assign
            | Self::EqualAssign
            | Self::SuperAssign
            | Self::Block
            | Self::Subset
            | Self::Extract
            | Self::Dollar
            | Self::Subassign
            | Self::ExtractAssign
            | Self::DollarAssign
            | Self::AndScalar
            | Self::OrScalar => SexpType::Special,
        }
    }
}
