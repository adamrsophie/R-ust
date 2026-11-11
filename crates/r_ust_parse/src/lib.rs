use r_ust_sexp::{BinaryOp, CallArgument, FunctionParameter, Sexp, UnaryOp, NA_REAL};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i32)]
pub enum ParseStatus {
    Null = 0,
    Ok = 1,
    Incomplete = 2,
    Error = 3,
    Eof = 4,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ParseError {
    pub status: ParseStatus,
    pub message: String,
}

impl ParseError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            status: ParseStatus::Error,
            message: message.into(),
        }
    }

    pub fn incomplete(message: impl Into<String>) -> Self {
        Self {
            status: ParseStatus::Incomplete,
            message: message.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ParseResult {
    pub status: ParseStatus,
    pub expressions: Vec<Sexp>,
}

#[derive(Default)]
pub struct Parser;

impl Parser {
    pub fn parse_all(&self, source: &str) -> Result<ParseResult, ParseError> {
        if source.trim().is_empty() {
            return Ok(ParseResult {
                status: ParseStatus::Null,
                expressions: Vec::new(),
            });
        }

        let tokens = Lexer::new(source).tokenize()?;
        let mut parser = SyntaxParser {
            tokens,
            position: 0,
        };
        let expressions = parser.parse_program()?;
        Ok(ParseResult {
            status: if expressions.is_empty() {
                ParseStatus::Null
            } else {
                ParseStatus::Ok
            },
            expressions,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
enum Token {
    Number(f64),
    Integer(i32),
    String(String),
    Ident(String),
    Plus,
    Minus,
    Star,
    Slash,
    Caret,
    Bang,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Colon,
    Semicolon,
    AssignLeft,
    AssignLeftSuper,
    AssignRight,
    AssignRightSuper,
    AssignEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    EqualEqual,
    NotEqual,
    Ampersand,
    AndAnd,
    Pipe,
    OrOr,
    Dollar,
    Special(String),
    Eof,
}

struct Lexer<'a> {
    source: &'a str,
    chars: Vec<char>,
    position: usize,
}

impl<'a> Lexer<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            chars: source.chars().collect(),
            position: 0,
        }
    }

    fn tokenize(mut self) -> Result<Vec<Token>, ParseError> {
        let mut tokens = Vec::new();
        while let Some(ch) = self.peek() {
            match ch {
                ' ' | '\t' | '\r' => {
                    self.advance();
                }
                '\n' | ';' => {
                    self.advance();
                    tokens.push(Token::Semicolon);
                }
                '#' => {
                    self.skip_comment();
                }
                '0'..='9' | '.' if self.starts_number() => {
                    tokens.push(self.lex_number()?);
                }
                '"' | '\'' => {
                    tokens.push(Token::String(self.lex_string()?));
                }
                'A'..='Z' | 'a'..='z' | '_' | '.' => {
                    tokens.push(Token::Ident(self.lex_identifier()));
                }
                '+' => {
                    self.advance();
                    tokens.push(Token::Plus);
                }
                '-' => {
                    self.advance();
                    if self.consume('>') {
                        if self.consume('>') {
                            tokens.push(Token::AssignRightSuper);
                        } else {
                            tokens.push(Token::AssignRight);
                        }
                    } else {
                        tokens.push(Token::Minus);
                    }
                }
                '*' => {
                    self.advance();
                    tokens.push(if self.consume('*') {
                        Token::Caret
                    } else {
                        Token::Star
                    });
                }
                '/' => {
                    self.advance();
                    tokens.push(Token::Slash);
                }
                '^' => {
                    self.advance();
                    tokens.push(Token::Caret);
                }
                '(' => {
                    self.advance();
                    tokens.push(Token::LParen);
                }
                ')' => {
                    self.advance();
                    tokens.push(Token::RParen);
                }
                '{' => {
                    self.advance();
                    tokens.push(Token::LBrace);
                }
                '}' => {
                    self.advance();
                    tokens.push(Token::RBrace);
                }
                '[' => {
                    self.advance();
                    tokens.push(Token::LBracket);
                }
                ']' => {
                    self.advance();
                    tokens.push(Token::RBracket);
                }
                ',' => {
                    self.advance();
                    tokens.push(Token::Comma);
                }
                ':' => {
                    self.advance();
                    tokens.push(Token::Colon);
                }
                '<' => {
                    self.advance();
                    if self.consume('<') {
                        if self.consume('-') {
                            tokens.push(Token::AssignLeftSuper);
                        } else {
                            return Err(ParseError::new(format!(
                                "unexpected operator << at byte {}",
                                self.byte_position()
                            )));
                        }
                    } else if self.consume('-') {
                        tokens.push(Token::AssignLeft);
                    } else if self.consume('=') {
                        tokens.push(Token::LessEqual);
                    } else {
                        tokens.push(Token::Less);
                    }
                }
                '>' => {
                    self.advance();
                    if self.consume('=') {
                        tokens.push(Token::GreaterEqual);
                    } else {
                        tokens.push(Token::Greater);
                    }
                }
                '=' => {
                    self.advance();
                    if self.consume('=') {
                        tokens.push(Token::EqualEqual);
                    } else {
                        tokens.push(Token::AssignEqual);
                    }
                }
                '!' => {
                    self.advance();
                    if self.consume('=') {
                        tokens.push(Token::NotEqual);
                    } else {
                        tokens.push(Token::Bang);
                    }
                }
                '&' => {
                    self.advance();
                    if self.consume('&') {
                        tokens.push(Token::AndAnd);
                    } else {
                        tokens.push(Token::Ampersand);
                    }
                }
                '|' => {
                    self.advance();
                    if self.consume('|') {
                        tokens.push(Token::OrOr);
                    } else {
                        tokens.push(Token::Pipe);
                    }
                }
                '$' => {
                    self.advance();
                    tokens.push(Token::Dollar);
                }
                '%' => {
                    tokens.push(Token::Special(self.lex_special_operator()?));
                }
                _ => {
                    return Err(ParseError::new(format!(
                        "unexpected character {:?} at byte {}",
                        ch,
                        self.byte_position()
                    )));
                }
            }
        }
        tokens.push(Token::Eof);
        Ok(tokens)
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.position).copied()
    }

    fn peek_next(&self) -> Option<char> {
        self.chars.get(self.position + 1).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.position += 1;
        Some(ch)
    }

    fn consume(&mut self, expected: char) -> bool {
        if self.peek() == Some(expected) {
            self.position += 1;
            true
        } else {
            false
        }
    }

    fn skip_comment(&mut self) {
        while let Some(ch) = self.peek() {
            self.advance();
            if ch == '\n' {
                break;
            }
        }
    }

    fn starts_number(&self) -> bool {
        match self.peek() {
            Some('0'..='9') => true,
            Some('.') => matches!(self.peek_next(), Some('0'..='9')),
            _ => false,
        }
    }

    fn lex_number(&mut self) -> Result<Token, ParseError> {
        let start = self.position;
        while matches!(self.peek(), Some('0'..='9')) {
            self.advance();
        }
        if self.peek() == Some('.') {
            self.advance();
            while matches!(self.peek(), Some('0'..='9')) {
                self.advance();
            }
        }
        if matches!(self.peek(), Some('e' | 'E')) {
            self.advance();
            if matches!(self.peek(), Some('+' | '-')) {
                self.advance();
            }
            let exp_start = self.position;
            while matches!(self.peek(), Some('0'..='9')) {
                self.advance();
            }
            if exp_start == self.position {
                return Err(ParseError::new("invalid numeric exponent"));
            }
        }
        let text: String = self.chars[start..self.position].iter().collect();
        let value = text
            .parse::<f64>()
            .map_err(|_| ParseError::new(format!("invalid numeric literal {text:?}")))?;
        if self.consume('L') {
            if value.fract() == 0.0 && value <= i32::MAX as f64 {
                return Ok(Token::Integer(value as i32));
            }
            // shortcut: invalid L qualifiers use double silently until parser warnings exist.
        }
        Ok(Token::Number(value))
    }

    fn lex_string(&mut self) -> Result<String, ParseError> {
        let quote = self.advance().expect("string opener");
        let mut out = String::new();
        while let Some(ch) = self.advance() {
            if ch == quote {
                return Ok(out);
            }
            if ch == '\\' {
                let escaped = self
                    .advance()
                    .ok_or_else(|| ParseError::incomplete("unterminated string escape"))?;
                match escaped {
                    'n' => out.push('\n'),
                    'r' => out.push('\r'),
                    't' => out.push('\t'),
                    '\\' => out.push('\\'),
                    '\'' => out.push('\''),
                    '"' => out.push('"'),
                    other => out.push(other),
                }
            } else {
                out.push(ch);
            }
        }
        Err(ParseError::incomplete("unterminated string literal"))
    }

    fn lex_identifier(&mut self) -> String {
        let start = self.position;
        self.advance();
        while matches!(
            self.peek(),
            Some('A'..='Z' | 'a'..='z' | '0'..='9' | '_' | '.')
        ) {
            self.advance();
        }
        self.chars[start..self.position].iter().collect()
    }

    fn lex_special_operator(&mut self) -> Result<String, ParseError> {
        self.advance();
        let start = self.position;
        while let Some(ch) = self.peek() {
            if ch == '%' {
                let text: String = self.chars[start..self.position].iter().collect();
                self.advance();
                return Ok(text);
            }
            self.advance();
        }
        Err(ParseError::incomplete("unterminated special operator"))
    }

    fn byte_position(&self) -> usize {
        self.source
            .char_indices()
            .nth(self.position)
            .map(|(index, _)| index)
            .unwrap_or_else(|| self.source.len())
    }
}

struct SyntaxParser {
    tokens: Vec<Token>,
    position: usize,
}

impl SyntaxParser {
    fn parse_program(&mut self) -> Result<Vec<Sexp>, ParseError> {
        let mut expressions = Vec::new();
        self.consume_separators();
        while !matches!(self.peek(), Token::Eof) {
            expressions.push(self.parse_expression()?);
            if matches!(self.peek(), Token::Eof) {
                break;
            }
            if !self.consume_separators() {
                return Err(ParseError::new("expected expression separator"));
            }
        }
        Ok(expressions)
    }

    fn parse_expression(&mut self) -> Result<Sexp, ParseError> {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> Result<Sexp, ParseError> {
        let left = self.parse_binary(0)?;
        match self.peek() {
            Token::AssignLeft | Token::AssignEqual | Token::AssignLeftSuper => {
                let is_super = matches!(self.peek(), Token::AssignLeftSuper);
                self.advance();
                let value = self.parse_assignment()?;
                return Self::assignment_from_target(left, value, is_super);
            }
            Token::AssignRight | Token::AssignRightSuper => {
                let is_super = matches!(self.peek(), Token::AssignRightSuper);
                self.advance();
                let target = self.parse_assignment()?;
                return Self::assignment_from_target(target, left, is_super);
            }
            _ => {}
        }
        Ok(left)
    }

    fn assignment_from_target(
        target_expr: Sexp,
        value: Sexp,
        is_super: bool,
    ) -> Result<Sexp, ParseError> {
        if is_super {
            return match target_expr {
                Sexp::Symbol(target) => Ok(Sexp::SuperAssignment {
                    target,
                    value: Box::new(value),
                }),
                _ => Err(ParseError::new(
                    "superassignment currently supports only symbol targets",
                )),
            };
        }

        match target_expr {
            Sexp::Symbol(target) => Ok(Sexp::Assignment {
                target,
                value: Box::new(value),
            }),
            Sexp::Subset {
                value: target,
                index,
            } => match *target {
                Sexp::Symbol(target) => Ok(Sexp::Subassignment {
                    target,
                    index,
                    value: Box::new(value),
                }),
                _ => Err(ParseError::new(
                    "left side of subset assignment must target a name",
                )),
            },
            Sexp::Subset2D {
                value: target,
                row,
                col,
            } => match *target {
                Sexp::Symbol(target) => Ok(Sexp::MatrixSubassignment {
                    target,
                    row,
                    col,
                    value: Box::new(value),
                }),
                _ => Err(ParseError::new(
                    "left side of matrix assignment must target a name",
                )),
            },
            Sexp::Extract {
                value: target,
                index,
            } => match *target {
                Sexp::Symbol(target) => Ok(Sexp::ExtractAssignment {
                    target,
                    index,
                    value: Box::new(value),
                }),
                _ => Err(ParseError::new(
                    "left side of extract assignment must target a name",
                )),
            },
            Sexp::Dollar {
                value: target,
                name,
            } => match *target {
                Sexp::Symbol(target) => Ok(Sexp::DollarAssignment {
                    target,
                    name,
                    value: Box::new(value),
                }),
                _ => Err(ParseError::new(
                    "left side of dollar assignment must target a name",
                )),
            },
            Sexp::Call {
                function,
                arguments,
            } => match *function {
                Sexp::Symbol(function) if function == "names" => {
                    let mut arguments = arguments.into_iter();
                    let Some(argument) = arguments.next() else {
                        return Err(ParseError::new("names assignment needs a target"));
                    };
                    if arguments.next().is_some() || argument.name.is_some() {
                        return Err(ParseError::new(
                            "names assignment must target one positional name",
                        ));
                    }
                    match argument.value {
                        Sexp::Symbol(target) => Ok(Sexp::NamesAssignment {
                            target,
                            value: Box::new(value),
                        }),
                        _ => Err(ParseError::new("names assignment must target a name")),
                    }
                }
                _ => Err(ParseError::new(
                    "left side of assignment must be a name, subset, or replacement call",
                )),
            },
            _ => Err(ParseError::new(
                "left side of assignment must be a name or subset",
            )),
        }
    }

    fn parse_binary(&mut self, min_precedence: u8) -> Result<Sexp, ParseError> {
        let mut expr = self.parse_unary()?;
        while let Some((op, precedence)) = self.binary_operator()? {
            if precedence < min_precedence {
                break;
            }
            self.advance();
            let right = self.parse_binary(precedence + u8::from(op != BinaryOp::Power))?;
            expr = Sexp::Binary {
                op,
                left: Box::new(expr),
                right: Box::new(right),
            };
            if precedence == 7 && matches!(self.binary_operator()?, Some((_, 7))) {
                return Err(ParseError::new("comparison operators are not associative"));
            }
        }
        Ok(expr)
    }

    fn binary_operator(&self) -> Result<Option<(BinaryOp, u8)>, ParseError> {
        // R's gram.y: powers bind above signs; comparisons bind above !.
        Ok(Some(match self.peek() {
            Token::OrOr => (BinaryOp::OrScalar, 2),
            Token::Pipe => (BinaryOp::Or, 2),
            Token::AndAnd => (BinaryOp::AndScalar, 4),
            Token::Ampersand => (BinaryOp::And, 4),
            Token::Less => (BinaryOp::Less, 7),
            Token::LessEqual => (BinaryOp::LessEqual, 7),
            Token::Greater => (BinaryOp::Greater, 7),
            Token::GreaterEqual => (BinaryOp::GreaterEqual, 7),
            Token::EqualEqual => (BinaryOp::Equal, 7),
            Token::NotEqual => (BinaryOp::NotEqual, 7),
            Token::Plus => (BinaryOp::Add, 8),
            Token::Minus => (BinaryOp::Subtract, 8),
            Token::Star => (BinaryOp::Multiply, 9),
            Token::Slash => (BinaryOp::Divide, 9),
            Token::Special(name) if name == "in" => (BinaryOp::In, 10),
            Token::Special(name) => {
                return Err(ParseError::new(format!(
                    "special operator %{name}% is not implemented"
                )))
            }
            Token::Colon => (BinaryOp::Sequence, 11),
            Token::Caret => (BinaryOp::Power, 13),
            _ => return Ok(None),
        }))
    }

    fn parse_unary(&mut self) -> Result<Sexp, ParseError> {
        let (op, precedence) = match self.peek() {
            Token::Plus => (UnaryOp::Plus, 12),
            Token::Minus => (UnaryOp::Minus, 12),
            Token::Bang => (UnaryOp::Not, 6),
            _ => return self.parse_call(),
        };
        self.advance();
        Ok(Sexp::Unary {
            op,
            value: Box::new(self.parse_binary(precedence)?),
        })
    }

    fn parse_call(&mut self) -> Result<Sexp, ParseError> {
        let mut expr = self.parse_primary()?;
        loop {
            match self.peek() {
                Token::LParen => {
                    self.advance();
                    let arguments = self.parse_call_arguments()?;
                    self.expect(Token::RParen)?;
                    expr = Sexp::Call {
                        function: Box::new(expr),
                        arguments,
                    };
                }
                Token::LBracket => {
                    self.advance();
                    if matches!(self.peek(), Token::LBracket) {
                        self.advance();
                        let index = self.parse_expression()?;
                        if matches!(self.peek(), Token::Comma) {
                            return Err(ParseError::new(
                                "only one-dimensional extraction is implemented",
                            ));
                        }
                        self.expect(Token::RBracket)?;
                        self.expect(Token::RBracket)?;
                        expr = Sexp::Extract {
                            value: Box::new(expr),
                            index: Box::new(index),
                        };
                        continue;
                    }
                    let index = if matches!(self.peek(), Token::RBracket | Token::Comma) {
                        None
                    } else {
                        Some(Box::new(self.parse_expression()?))
                    };
                    if matches!(self.peek(), Token::Comma) {
                        self.advance();
                        let col = if matches!(self.peek(), Token::RBracket) {
                            None
                        } else {
                            Some(Box::new(self.parse_expression()?))
                        };
                        self.expect(Token::RBracket)?;
                        expr = Sexp::Subset2D {
                            value: Box::new(expr),
                            row: index,
                            col,
                        };
                        continue;
                    }
                    self.expect(Token::RBracket)?;
                    expr = Sexp::Subset {
                        value: Box::new(expr),
                        index,
                    };
                }
                Token::Dollar => {
                    self.advance();
                    let Token::Ident(name) = self.advance().clone() else {
                        return Err(ParseError::new("expected name after $"));
                    };
                    expr = Sexp::Dollar {
                        value: Box::new(expr),
                        name,
                    };
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn parse_call_arguments(&mut self) -> Result<Vec<CallArgument>, ParseError> {
        let mut arguments = Vec::new();
        if matches!(self.peek(), Token::RParen) {
            return Ok(arguments);
        }
        loop {
            let argument = if let Token::Ident(name) = self.peek().clone() {
                if matches!(self.peek_n(1), Some(Token::AssignEqual)) {
                    self.advance();
                    self.advance();
                    CallArgument::named(name, self.parse_expression()?)
                } else {
                    CallArgument::positional(self.parse_expression()?)
                }
            } else {
                CallArgument::positional(self.parse_expression()?)
            };
            arguments.push(argument);
            if !matches!(self.peek(), Token::Comma) {
                break;
            }
            self.advance();
            if matches!(self.peek(), Token::RParen) {
                break;
            }
        }
        Ok(arguments)
    }

    fn parse_primary(&mut self) -> Result<Sexp, ParseError> {
        match self.advance().clone() {
            Token::Number(value) => Ok(Sexp::Real(vec![value])),
            Token::Integer(value) => Ok(Sexp::Integer(vec![Some(value)])),
            Token::String(value) => Ok(Sexp::String(vec![Some(value)])),
            Token::Ident(name) => Ok(match name.as_str() {
                "function" => self.parse_function_definition()?,
                "if" => self.parse_if_expression()?,
                "for" => self.parse_for_expression()?,
                "while" => self.parse_while_expression()?,
                "repeat" => self.parse_repeat_expression()?,
                "break" => Sexp::Break,
                "next" => Sexp::Next,
                "NULL" => Sexp::Nil,
                "TRUE" | "T" => Sexp::Logical(vec![Some(true)]),
                "FALSE" | "F" => Sexp::Logical(vec![Some(false)]),
                "NA" => Sexp::Logical(vec![None]),
                "NA_integer_" => Sexp::Integer(vec![None]),
                "NA_real_" => Sexp::Real(vec![NA_REAL]),
                "NA_character_" => Sexp::String(vec![None]),
                "NaN" => Sexp::Real(vec![f64::NAN]),
                "Inf" => Sexp::Real(vec![f64::INFINITY]),
                _ => Sexp::Symbol(name),
            }),
            Token::LParen => {
                let expr = self.parse_expression()?;
                self.expect(Token::RParen)?;
                Ok(expr)
            }
            Token::LBrace => self.parse_block_after_open(),
            Token::Eof => Err(ParseError::incomplete("unexpected end of input")),
            other => Err(ParseError::new(format!("unexpected token {other:?}"))),
        }
    }

    fn parse_function_definition(&mut self) -> Result<Sexp, ParseError> {
        self.expect(Token::LParen)?;
        let parameters = self.parse_function_parameters()?;
        self.expect(Token::RParen)?;
        let body = self.parse_expression()?;
        Ok(Sexp::Function {
            parameters,
            body: Box::new(body),
        })
    }

    fn parse_function_parameters(&mut self) -> Result<Vec<FunctionParameter>, ParseError> {
        let mut parameters = Vec::new();
        if matches!(self.peek(), Token::RParen) {
            return Ok(parameters);
        }
        loop {
            let Token::Ident(name) = self.advance().clone() else {
                return Err(ParseError::new("expected function parameter name"));
            };
            let parameter = if matches!(self.peek(), Token::AssignEqual) {
                self.advance();
                FunctionParameter::optional(name, self.parse_expression()?)
            } else {
                FunctionParameter::required(name)
            };
            parameters.push(parameter);
            if !matches!(self.peek(), Token::Comma) {
                break;
            }
            self.advance();
            if matches!(self.peek(), Token::RParen) {
                break;
            }
        }
        Ok(parameters)
    }

    fn parse_if_expression(&mut self) -> Result<Sexp, ParseError> {
        self.expect(Token::LParen)?;
        let condition = self.parse_expression()?;
        self.expect(Token::RParen)?;
        let then_branch = self.parse_expression()?;

        let saved = self.position;
        self.consume_separators();
        let else_branch = if matches!(self.peek(), Token::Ident(name) if name == "else") {
            self.advance();
            Some(Box::new(self.parse_expression()?))
        } else {
            self.position = saved;
            None
        };

        Ok(Sexp::If {
            condition: Box::new(condition),
            then_branch: Box::new(then_branch),
            else_branch,
        })
    }

    fn parse_for_expression(&mut self) -> Result<Sexp, ParseError> {
        self.expect(Token::LParen)?;
        let Token::Ident(variable) = self.advance().clone() else {
            return Err(ParseError::new("expected loop variable name"));
        };
        match self.advance().clone() {
            Token::Ident(keyword) if keyword == "in" => {}
            other => return Err(ParseError::new(format!("expected in, found {other:?}"))),
        }
        let sequence = self.parse_expression()?;
        self.expect(Token::RParen)?;
        let body = self.parse_expression()?;
        Ok(Sexp::For {
            variable,
            sequence: Box::new(sequence),
            body: Box::new(body),
        })
    }

    fn parse_while_expression(&mut self) -> Result<Sexp, ParseError> {
        self.expect(Token::LParen)?;
        let condition = self.parse_expression()?;
        self.expect(Token::RParen)?;
        let body = self.parse_expression()?;
        Ok(Sexp::While {
            condition: Box::new(condition),
            body: Box::new(body),
        })
    }

    fn parse_repeat_expression(&mut self) -> Result<Sexp, ParseError> {
        let body = self.parse_expression()?;
        Ok(Sexp::Repeat {
            body: Box::new(body),
        })
    }

    fn parse_block_after_open(&mut self) -> Result<Sexp, ParseError> {
        let mut expressions = Vec::new();
        self.consume_separators();
        while !matches!(self.peek(), Token::RBrace | Token::Eof) {
            expressions.push(self.parse_expression()?);
            if matches!(self.peek(), Token::RBrace) {
                break;
            }
            if !self.consume_separators() {
                return Err(ParseError::new("expected expression separator in block"));
            }
        }
        self.expect(Token::RBrace)?;
        Ok(Sexp::Block(expressions))
    }

    fn consume_separators(&mut self) -> bool {
        let mut consumed = false;
        while matches!(self.peek(), Token::Semicolon) {
            self.advance();
            consumed = true;
        }
        consumed
    }

    fn expect(&mut self, expected: Token) -> Result<(), ParseError> {
        let actual = self.advance().clone();
        if std::mem::discriminant(&actual) == std::mem::discriminant(&expected) {
            Ok(())
        } else {
            Err(ParseError::new(format!(
                "expected {expected:?}, found {actual:?}"
            )))
        }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.position]
    }

    fn peek_n(&self, offset: usize) -> Option<&Token> {
        self.tokens.get(self.position + offset)
    }

    fn advance(&mut self) -> &Token {
        let token = &self.tokens[self.position];
        if !matches!(token, Token::Eof) {
            self.position += 1;
        }
        token
    }
}

#[cfg(test)]
mod tests {
    use super::{ParseStatus, Parser};
    use r_ust_sexp::{BinaryOp, Sexp};

    #[test]
    fn parses_assignment_and_precedence() {
        let parsed = Parser.parse_all("x <- 1 + 2 * 3").unwrap();
        assert_eq!(parsed.status, ParseStatus::Ok);
        assert_eq!(parsed.expressions.len(), 1);
        let Sexp::Assignment { target, value } = &parsed.expressions[0] else {
            panic!("expected assignment");
        };
        assert_eq!(target, "x");
        let Sexp::Binary { op, .. } = value.as_ref() else {
            panic!("expected binary expression");
        };
        assert_eq!(*op, BinaryOp::Add);
    }

    #[test]
    fn parses_named_call_arguments() {
        let parsed = Parser.parse_all("dnorm(0, mean = 0, sd = 1)").unwrap();
        let Sexp::Call { arguments, .. } = &parsed.expressions[0] else {
            panic!("expected call");
        };
        assert_eq!(arguments.len(), 3);
        assert_eq!(arguments[1].name.as_deref(), Some("mean"));
        assert_eq!(arguments[2].name.as_deref(), Some("sd"));
    }

    #[test]
    fn rejects_unparenthesized_comparison_chains() {
        for source in ["1 < 2 < 3", "1 == 1 != 2", "1 <= 2 > 0"] {
            assert!(Parser.parse_all(source).is_err(), "{source}");
        }
        for source in ["(1 < 2) < 3", "1 == (1 != 2)"] {
            assert!(Parser.parse_all(source).is_ok(), "{source}");
        }
    }

    #[test]
    fn parses_function_if_and_block() {
        let parsed = Parser
            .parse_all("f <- function(x, y = 1) { if (x > 0) x + y else y }")
            .unwrap();
        let Sexp::Assignment { value, .. } = &parsed.expressions[0] else {
            panic!("expected assignment");
        };
        let Sexp::Function { parameters, body } = value.as_ref() else {
            panic!("expected function");
        };
        assert_eq!(parameters.len(), 2);
        assert!(parameters[1].default.is_some());
        assert!(matches!(body.as_ref(), Sexp::Block(_)));
    }

    #[test]
    fn parses_loop_sequence_and_subset() {
        let parsed = Parser
            .parse_all("for (i in 1:3) { x <- c(10, 20, 30)[i] }")
            .unwrap();
        assert!(matches!(parsed.expressions[0], Sexp::For { .. }));
    }

    #[test]
    fn parses_subset_assignment() {
        let parsed = Parser.parse_all("x[2] <- 99").unwrap();
        let Sexp::Subassignment {
            target,
            index,
            value,
        } = &parsed.expressions[0]
        else {
            panic!("expected subset assignment");
        };
        assert_eq!(target, "x");
        assert!(index.is_some());
        assert_eq!(value.as_ref(), &Sexp::Real(vec![99.0]));
    }

    #[test]
    fn parses_logical_operators() {
        let parsed = Parser.parse_all("TRUE || FALSE & FALSE && TRUE").unwrap();
        let Sexp::Binary { op, right, .. } = &parsed.expressions[0] else {
            panic!("expected binary expression");
        };
        assert_eq!(*op, BinaryOp::OrScalar);
        let Sexp::Binary { op, .. } = right.as_ref() else {
            panic!("expected nested binary expression");
        };
        assert_eq!(*op, BinaryOp::AndScalar);
    }

    #[test]
    fn parses_list_extraction_forms() {
        let parsed = Parser.parse_all("x[[2]]; x$name").unwrap();
        assert!(matches!(parsed.expressions[0], Sexp::Extract { .. }));
        let Sexp::Dollar { name, .. } = &parsed.expressions[1] else {
            panic!("expected dollar expression");
        };
        assert_eq!(name, "name");
    }

    #[test]
    fn parses_in_operator() {
        let parsed = Parser.parse_all("x %in% y").unwrap();
        let Sexp::Binary { op, .. } = &parsed.expressions[0] else {
            panic!("expected binary expression");
        };
        assert_eq!(*op, BinaryOp::In);
    }

    #[test]
    fn parses_extract_and_dollar_assignment() {
        let parsed = Parser.parse_all("x[[2]] <- 3; x$name <- 4").unwrap();
        assert!(matches!(
            parsed.expressions[0],
            Sexp::ExtractAssignment { .. }
        ));
        let Sexp::DollarAssignment { name, .. } = &parsed.expressions[1] else {
            panic!("expected dollar assignment");
        };
        assert_eq!(name, "name");
    }

    #[test]
    fn parses_matrix_subset_and_assignment() {
        let parsed = Parser.parse_all("m[1, 2]; m[, 1] <- 9").unwrap();
        let Sexp::Subset2D { row, col, .. } = &parsed.expressions[0] else {
            panic!("expected matrix subset");
        };
        assert!(row.is_some());
        assert!(col.is_some());
        let Sexp::MatrixSubassignment { row, col, .. } = &parsed.expressions[1] else {
            panic!("expected matrix subassignment");
        };
        assert!(row.is_none());
        assert!(col.is_some());
    }

    #[test]
    fn parses_names_assignment() {
        let parsed = Parser.parse_all("names(x) <- c(\"a\", \"b\")").unwrap();
        let Sexp::NamesAssignment { target, value } = &parsed.expressions[0] else {
            panic!("expected names assignment");
        };
        assert_eq!(target, "x");
        assert!(matches!(value.as_ref(), Sexp::Call { .. }));
    }

    #[test]
    fn parses_right_and_super_assignment() {
        let parsed = Parser.parse_all("1 + 2 -> x; x <<- 4; 5 ->> y").unwrap();
        let Sexp::Assignment { target, value } = &parsed.expressions[0] else {
            panic!("expected right assignment");
        };
        assert_eq!(target, "x");
        assert!(matches!(value.as_ref(), Sexp::Binary { .. }));
        let Sexp::SuperAssignment { target, value } = &parsed.expressions[1] else {
            panic!("expected superassignment");
        };
        assert_eq!(target, "x");
        assert_eq!(value.as_ref(), &Sexp::Real(vec![4.0]));
        let Sexp::SuperAssignment { target, value } = &parsed.expressions[2] else {
            panic!("expected right superassignment");
        };
        assert_eq!(target, "y");
        assert_eq!(value.as_ref(), &Sexp::Real(vec![5.0]));
    }
}
