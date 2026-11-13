use r_ust_sexp::{BinaryOp, CallArgument, FunctionParameter, Sexp, UnaryOp, NA_REAL};

mod operators;

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
    pub warnings: Vec<String>,
}

impl ParseError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            status: ParseStatus::Error,
            message: message.into(),
            warnings: Vec::new(),
        }
    }

    pub fn incomplete(message: impl Into<String>) -> Self {
        Self {
            status: ParseStatus::Incomplete,
            message: message.into(),
            warnings: Vec::new(),
        }
    }

    fn with_warnings(mut self, warnings: Vec<String>) -> Self {
        self.warnings = warnings;
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ParseResult {
    pub status: ParseStatus,
    pub expressions: Vec<Sexp>,
    pub warnings: Vec<String>,
}

#[derive(Default)]
pub struct Parser;

impl Parser {
    /// Parses complete statements lazily; later text is not tokenized until requested.
    pub fn parse_iter<'a>(&self, source: &'a str) -> ParseIter<'a> {
        ParseIter {
            lexer: Lexer::new(source),
            consumed_bytes: 0,
            finished: false,
        }
    }

    /// Parses arriving input; a final statement without a separator remains incomplete.
    pub fn parse_iter_open<'a>(&self, source: &'a str) -> ParseIter<'a> {
        let mut statements = self.parse_iter(source);
        statements.lexer.open_input = true;
        statements
    }

    pub fn parse_all(&self, source: &str) -> Result<ParseResult, ParseError> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer
            .tokenize()
            .map_err(|error| error.with_warnings(std::mem::take(&mut lexer.warnings)))?;
        let mut parser = SyntaxParser {
            tokens,
            position: 0,
            block_depth: 0,
        };
        let expressions = parser
            .parse_program()
            .map_err(|error| error.with_warnings(std::mem::take(&mut lexer.warnings)))?;
        Ok(ParseResult {
            status: if expressions.is_empty() {
                ParseStatus::Null
            } else {
                ParseStatus::Ok
            },
            expressions,
            warnings: lexer.warnings,
        })
    }
}

pub struct ParseIter<'a> {
    lexer: Lexer<'a>,
    consumed_bytes: usize,
    finished: bool,
}

impl ParseIter<'_> {
    /// Bytes through the last complete statement, excluding a failing or incomplete statement.
    pub fn consumed_bytes(&self) -> usize {
        self.consumed_bytes
    }
}

impl Iterator for ParseIter<'_> {
    type Item = Result<ParseResult, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished || self.lexer.peek().is_none() {
            self.finished = true;
            return None;
        }
        let mut tokens = Vec::new();
        loop {
            match self.lexer.tokenize_chunk(true) {
                Ok(chunk) => tokens.extend(chunk),
                Err(error) => {
                    self.finished = true;
                    return Some(Err(
                        error.with_warnings(std::mem::take(&mut self.lexer.warnings))
                    ));
                }
            }
            let terminated = matches!(
                tokens.iter().rev().nth(1),
                Some(Token::Newline | Token::Semicolon)
            );
            let mut parser = SyntaxParser {
                tokens,
                position: 0,
                block_depth: 0,
            };
            match parser.parse_program() {
                Ok(expressions) => {
                    if self.lexer.open_input
                        && self.lexer.peek().is_none()
                        && !terminated
                        && (!expressions.is_empty() || self.lexer.unterminated_comment)
                    {
                        self.finished = true;
                        return Some(Err(ParseError::incomplete("statement needs more input")
                            .with_warnings(std::mem::take(&mut self.lexer.warnings))));
                    }
                    self.consumed_bytes = self.lexer.byte_position();
                    return Some(Ok(ParseResult {
                        status: if expressions.is_empty() {
                            ParseStatus::Null
                        } else {
                            ParseStatus::Ok
                        },
                        expressions,
                        warnings: std::mem::take(&mut self.lexer.warnings),
                    }));
                }
                Err(error)
                    if error.status == ParseStatus::Incomplete && self.lexer.peek().is_some() =>
                {
                    tokens = parser.tokens;
                    tokens.pop(); // Replace the temporary end marker when adding the continuation.
                }
                Err(error) => {
                    self.finished = true;
                    return Some(Err(
                        error.with_warnings(std::mem::take(&mut self.lexer.warnings))
                    ));
                }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum Token {
    Number(f64),
    Integer(i32),
    String(String),
    Ident(String),
    QuotedIdent(String),
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
    Newline,
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
    warnings: Vec<String>,
    open_input: bool,
    unterminated_comment: bool,
}

impl<'a> Lexer<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            chars: source.chars().collect(),
            position: 0,
            warnings: Vec::new(),
            open_input: false,
            unterminated_comment: false,
        }
    }

    fn tokenize(&mut self) -> Result<Vec<Token>, ParseError> {
        self.tokenize_chunk(false)
    }

    fn missing_token(&self, message: impl Into<String>) -> ParseError {
        if self.open_input && self.peek().is_none() {
            ParseError::incomplete(message)
        } else {
            ParseError::new(message)
        }
    }

    fn tokenize_chunk(&mut self, stop_at_statement: bool) -> Result<Vec<Token>, ParseError> {
        let mut tokens = Vec::new();
        let mut groups = Vec::new();
        while let Some(ch) = self.peek() {
            match ch {
                ' '
                | '\t'
                | '\x0c'
                | '\u{1680}'
                | '\u{2000}'..='\u{2006}'
                | '\u{2008}'..='\u{200a}'
                | '\u{205f}'
                | '\u{3000}' => {
                    self.advance();
                }
                '\r' if self.peek_next() == Some('\n') => {
                    self.advance();
                }
                '\n' => {
                    self.advance();
                    if !matches!(groups.last(), Some('(' | '[')) {
                        tokens.push(Token::Newline);
                    }
                }
                ';' => {
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
                '`' => {
                    tokens.push(Token::QuotedIdent(self.lex_string()?));
                }
                'r' | 'R' if matches!(self.peek_next(), Some('"' | '\'')) => {
                    tokens.push(Token::String(self.lex_raw_string()?));
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
                    groups.push('(');
                    tokens.push(Token::LParen);
                }
                ')' => {
                    self.advance();
                    groups.pop();
                    tokens.push(Token::RParen);
                }
                '{' => {
                    self.advance();
                    groups.push('{');
                    tokens.push(Token::LBrace);
                }
                '}' => {
                    self.advance();
                    groups.pop();
                    tokens.push(Token::RBrace);
                }
                '[' => {
                    self.advance();
                    groups.push('[');
                    tokens.push(Token::LBracket);
                }
                ']' => {
                    self.advance();
                    groups.pop();
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
                            return Err(self.missing_token(format!(
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
            if stop_at_statement
                && groups.is_empty()
                && matches!(tokens.last(), Some(Token::Newline | Token::Semicolon))
            {
                break;
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
            if ch == '\n' {
                break;
            }
            self.advance();
        }
        self.unterminated_comment = self.peek().is_none();
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
        let hexadecimal = self.peek() == Some('0') && matches!(self.peek_next(), Some('x' | 'X'));
        let value = if hexadecimal {
            self.lex_hex_number()?
        } else {
            self.lex_decimal_number()?
        };
        if self.consume('L') {
            let literal: String = self.chars[start..self.position].iter().collect();
            let integer = value.fract() == 0.0 && value <= i32::MAX as f64;
            let decimal = literal.contains('.')
                && !literal.contains(if hexadecimal { ['p', 'P'] } else { ['e', 'E'] });
            if !integer {
                self.warnings.push(if decimal {
                    format!("integer literal {literal} contains decimal; using numeric value")
                } else {
                    format!("non-integer value {literal} qualified with L; using numeric value")
                });
            } else {
                if decimal {
                    self.warnings.push(format!(
                        "integer literal {literal} contains unnecessary decimal point"
                    ));
                }
                return Ok(Token::Integer(value as i32));
            }
        }
        Ok(Token::Number(value))
    }

    fn lex_decimal_number(&mut self) -> Result<f64, ParseError> {
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
                return Err(self.missing_token("invalid numeric exponent"));
            }
        }
        let text: String = self.chars[start..self.position].iter().collect();
        text.parse::<f64>()
            .map_err(|_| ParseError::new(format!("invalid numeric literal {text:?}")))
    }

    fn lex_hex_number(&mut self) -> Result<f64, ParseError> {
        self.position += 2;
        let start = self.position;
        let mut seen_dot = false;
        let mut fractional_bits = 0_i64;
        // shortcut: accumulation matches R with 64-bit LDOUBLE; use wider arithmetic for wider-LDOUBLE hosts.
        let mut value = 0.0;
        while let Some(ch) = self.peek() {
            if ch == '.' {
                if seen_dot {
                    return Err(ParseError::new("invalid hexadecimal decimal point"));
                }
                seen_dot = true;
            } else if let Some(digit) = ch.to_digit(16) {
                value = 16.0 * value + f64::from(digit);
                if seen_dot {
                    fractional_bits += 4;
                }
            } else {
                break;
            }
            self.advance();
        }
        if self.position == start {
            return Err(self.missing_token("invalid hexadecimal literal"));
        }
        let mut exponent = 0_i64;
        if matches!(self.peek(), Some('p' | 'P')) {
            self.advance();
            let negative = self.consume('-');
            if !negative {
                self.consume('+');
            }
            let start = self.position;
            while let Some(ch @ '0'..='9') = self.peek() {
                if exponent < 9999 {
                    exponent = exponent * 10 + i64::from(ch as u8 - b'0');
                }
                self.advance();
            }
            if self.position == start {
                return Err(self.missing_token("invalid hexadecimal exponent"));
            }
            if negative {
                exponent = -exponent;
            }
        }
        if value != 0.0 {
            if fractional_bits > 0 {
                if exponent - fractional_bits < -122 {
                    value /= Self::binary_factor(fractional_bits as u64);
                } else {
                    exponent -= fractional_bits;
                }
            }
            let factor = Self::binary_factor(exponent.unsigned_abs());
            value = if exponent < 0 {
                value / factor
            } else {
                value * factor
            };
        }
        Ok(value)
    }

    fn binary_factor(exponent: u64) -> f64 {
        if exponent >= 1024 {
            f64::INFINITY
        } else {
            f64::from_bits((exponent + 1023) << 52)
        }
    }

    fn lex_raw_string(&mut self) -> Result<String, ParseError> {
        self.advance();
        let quote = self.advance().expect("raw string quote");
        let mut dashes = 0;
        while self.consume('-') {
            dashes += 1;
        }
        let close = match self.advance() {
            Some('(') => ')',
            Some('[') => ']',
            Some('{') => '}',
            Some('|') => '|',
            _ => return Err(ParseError::new("malformed raw string opener")),
        };
        let mut value = String::new();
        while let Some(ch) = self.advance() {
            if ch == close {
                let checkpoint = self.position;
                if (0..dashes).all(|_| self.consume('-')) && self.consume(quote) {
                    return Ok(value);
                }
                self.position = checkpoint;
            }
            Self::validate_literal_character(ch)?;
            value.push(ch);
        }
        Err(ParseError::incomplete("unterminated raw string literal"))
    }

    fn validate_literal_character(ch: char) -> Result<(), ParseError> {
        if matches!(ch, '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}') {
            return Err(ParseError::new(
                "bidi formatting not allowed, use escapes instead",
            ));
        }
        if ch == '\0' {
            return Err(ParseError::new("nul character not allowed"));
        }
        Ok(())
    }

    fn lex_string(&mut self) -> Result<String, ParseError> {
        let quote = self.advance().expect("string opener");
        let mut bytes = Vec::new();
        let mut code_points: Vec<u32> = Vec::new();
        let mut has_unicode = false;
        let mut has_byte_escape = false;
        while let Some(ch) = self.advance() {
            if ch == quote {
                if quote == '`' {
                    if bytes.is_empty() {
                        return Err(ParseError::new("attempt to use zero-length variable name"));
                    }
                    if bytes.len() > 10000 {
                        return Err(ParseError::new("variable names are limited to 10000 bytes"));
                    }
                }
                if has_unicode {
                    if has_byte_escape {
                        return Err(ParseError::new(
                            "mixing Unicode and octal/hex escapes in a string is not allowed",
                        ));
                    }
                    if code_points.len() >= 9999 {
                        return Err(ParseError::new(
                            "string containing Unicode escapes is too long",
                        ));
                    }
                    let mut out = String::new();
                    let mut points = code_points.into_iter();
                    while let Some(mut point) = points.next() {
                        if (0xd800..=0xdbff).contains(&point) {
                            let low = points
                                .next()
                                .filter(|low| (0xdc00..=0xdfff).contains(low))
                                .ok_or_else(|| {
                                    ParseError::new(
                                        "unpaired surrogate strings are not supported yet",
                                    )
                                })?;
                            point = 0x10000 + ((point - 0xd800) << 10) + low - 0xdc00;
                        }
                        out.push(char::from_u32(point).ok_or_else(|| {
                            ParseError::new("unpaired surrogate strings are not supported yet")
                        })?);
                    }
                    return Ok(out);
                }
                // shortcut: string values require UTF-8; preserve raw bytes when string encodings are implemented.
                return String::from_utf8(bytes)
                    .map_err(|_| ParseError::new("non-UTF-8 byte strings are not supported yet"));
            }
            let decoded = if ch == '\\' {
                let escaped = self
                    .advance()
                    .ok_or_else(|| ParseError::incomplete("unterminated string escape"))?;
                match escaped {
                    'a' => '\x07',
                    'b' => '\x08',
                    'f' => '\x0c',
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    'v' => '\x0b',
                    '\\' | '\'' | '"' | '`' | ' ' | '\n' => escaped,
                    '0'..='7' | 'x' => {
                        let value = if escaped == 'x' {
                            let start = self.position;
                            let value = self.lex_escape_digits(16, 2, 0)?;
                            if self.position == start {
                                return Err(ParseError::new(
                                    "'\\x' used without hex digits in character string",
                                ));
                            }
                            value
                        } else {
                            self.lex_escape_digits(8, 2, escaped as u32 - '0' as u32)?
                        };
                        if value == 0 {
                            return Err(ParseError::new("nul character not allowed"));
                        }
                        if value > 255 {
                            return Err(ParseError::new(
                                "octal escape exceeds maximum allowed value \\377",
                            ));
                        }
                        bytes.push(value as u8);
                        has_byte_escape = true;
                        continue;
                    }
                    'u' | 'U' => {
                        if quote == '`' {
                            return Err(ParseError::new(
                                "Unicode escapes are not supported inside backticks",
                            ));
                        }
                        let braced = self.consume('{');
                        let start = self.position;
                        let value =
                            self.lex_escape_digits(16, if escaped == 'u' { 4 } else { 8 }, 0)?;
                        if self.position == start {
                            return Err(ParseError::new("Unicode escape used without hex digits"));
                        }
                        if braced && !self.consume('}') {
                            return Err(if self.peek().is_none() {
                                ParseError::incomplete("unterminated Unicode escape")
                            } else {
                                ParseError::new("invalid braced Unicode escape")
                            });
                        }
                        if value == 0 {
                            return Err(ParseError::new("nul character not allowed"));
                        }
                        if value > 0x10ffff {
                            return Err(ParseError::new("invalid Unicode escape value"));
                        }
                        code_points.push(value);
                        has_unicode = true;
                        continue;
                    }
                    other => {
                        return Err(ParseError::new(format!(
                            "'\\{other}' is an unrecognized escape in character string"
                        )))
                    }
                }
            } else {
                Self::validate_literal_character(ch)?;
                ch
            };
            let mut encoded = [0; 4];
            bytes.extend_from_slice(decoded.encode_utf8(&mut encoded).as_bytes());
            code_points.push(decoded as u32);
        }
        Err(ParseError::incomplete("unterminated string literal"))
    }

    fn lex_escape_digits(
        &mut self,
        radix: u32,
        limit: usize,
        mut value: u32,
    ) -> Result<u32, ParseError> {
        for _ in 0..limit {
            let ch = self
                .peek()
                .ok_or_else(|| ParseError::incomplete("unterminated string escape"))?;
            let Some(digit) = ch.to_digit(radix) else {
                break;
            };
            self.advance();
            value = value * radix + digit;
        }
        Ok(value)
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
    block_depth: usize,
}

impl SyntaxParser {
    fn parse_program(&mut self) -> Result<Vec<Sexp>, ParseError> {
        let mut expressions = Vec::new();
        while matches!(self.peek(), Token::Newline) {
            self.advance();
        }
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

    fn parse_unary(&mut self) -> Result<Sexp, ParseError> {
        self.consume_newlines();
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
                    let name = self.parse_name("expected name after $")?;
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
            let argument =
                if let Token::Ident(name) | Token::QuotedIdent(name) = self.peek().clone() {
                    if matches!(self.peek_n(1), Some(Token::AssignEqual)) {
                        self.advance();
                        self.advance();
                        let value = if matches!(self.peek(), Token::Comma | Token::RParen) {
                            Sexp::MissingArg(name.clone())
                        } else {
                            self.parse_expression()?
                        };
                        CallArgument::named(name, value)
                    } else {
                        CallArgument::positional(self.parse_expression()?)
                    }
                } else if matches!(self.peek(), Token::Comma | Token::RParen) {
                    CallArgument::positional(Sexp::MissingArg(String::new()))
                } else {
                    CallArgument::positional(self.parse_expression()?)
                };
            arguments.push(argument);
            if !matches!(self.peek(), Token::Comma) {
                break;
            }
            self.advance();
        }
        Ok(arguments)
    }

    fn parse_primary(&mut self) -> Result<Sexp, ParseError> {
        match self.advance().clone() {
            Token::Number(value) => Ok(Sexp::Real(vec![value])),
            Token::Integer(value) => Ok(Sexp::Integer(vec![Some(value)])),
            Token::String(value) => Ok(Sexp::String(vec![Some(value)])),
            Token::QuotedIdent(name) => Ok(Sexp::Symbol(name)),
            Token::Ident(name) => Ok(match name.as_str() {
                "function" => self.parse_function_definition()?,
                "if" => self.parse_if_expression()?,
                "for" => self.parse_for_expression()?,
                "while" => self.parse_while_expression()?,
                "repeat" => self.parse_repeat_expression()?,
                "else" => return Err(ParseError::new("unexpected 'else'")),
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
                Ok(Sexp::Parenthesized(Box::new(expr)))
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
            let name = self.parse_name("expected function parameter name")?;
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
        if self.block_depth > 0 {
            self.consume_newlines();
        }
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
        let variable = self.parse_name("expected loop variable name")?;
        match self.advance().clone() {
            Token::Ident(keyword) if keyword == "in" => {}
            Token::Eof => return Err(ParseError::incomplete("expected in")),
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
        self.block_depth += 1;
        let mut expressions = Vec::new();
        self.consume_separators();
        while !matches!(self.peek(), Token::RBrace | Token::Eof) {
            expressions.push(self.parse_expression()?);
            if matches!(self.peek(), Token::RBrace | Token::Eof) {
                break;
            }
            if !self.consume_separators() {
                return Err(ParseError::new("expected expression separator in block"));
            }
        }
        self.expect(Token::RBrace)?;
        self.block_depth -= 1;
        Ok(Sexp::Block(expressions))
    }

    fn consume_separators(&mut self) -> bool {
        let mut consumed = false;
        if matches!(self.peek(), Token::Semicolon) {
            self.advance();
            consumed = true;
        }
        while matches!(self.peek(), Token::Newline)
            || (self.block_depth > 0 && matches!(self.peek(), Token::Semicolon))
        {
            self.advance();
            consumed = true;
        }
        consumed
    }

    fn expect(&mut self, expected: Token) -> Result<(), ParseError> {
        self.consume_newlines();
        let actual = self.advance().clone();
        if std::mem::discriminant(&actual) == std::mem::discriminant(&expected) {
            Ok(())
        } else if matches!(actual, Token::Eof) {
            Err(ParseError::incomplete(format!("expected {expected:?}")))
        } else {
            Err(ParseError::new(format!(
                "expected {expected:?}, found {actual:?}"
            )))
        }
    }

    fn consume_newlines(&mut self) {
        while matches!(self.peek(), Token::Newline) {
            self.advance();
        }
    }

    fn parse_name(&mut self, message: &str) -> Result<String, ParseError> {
        self.consume_newlines();
        match self.advance().clone() {
            Token::Ident(name) | Token::QuotedIdent(name) => Ok(name),
            Token::Eof => Err(ParseError::incomplete(message)),
            _ => Err(ParseError::new(message)),
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
    fn omitted_call_arguments_keep_their_positions_and_names() {
        let parsed = Parser.parse_all("f(a=, ,3,)").unwrap();
        let Sexp::Call { arguments, .. } = &parsed.expressions[0] else {
            panic!("expected call");
        };
        assert_eq!(arguments.len(), 4);
        assert_eq!(arguments[0].name.as_deref(), Some("a"));
        assert!(matches!(&arguments[0].value, Sexp::MissingArg(name) if name == "a"));
        assert!(matches!(&arguments[1].value, Sexp::MissingArg(_)));
        assert!(matches!(&arguments[2].value, Sexp::Real(values) if values == &[3.0]));
        assert!(matches!(&arguments[3].value, Sexp::MissingArg(_)));
    }

    #[test]
    fn statement_iterator_stops_before_later_lexical_errors() {
        let first = "cat(\"é;😀\");";
        let source = format!("{first} @");
        let mut statements = Parser.parse_iter(&source);
        assert_eq!(statements.next().unwrap().unwrap().expressions.len(), 1);
        assert_eq!(statements.consumed_bytes(), first.len());
        assert_eq!(
            statements.next().unwrap().unwrap_err().status,
            ParseStatus::Error
        );
        assert_eq!(statements.consumed_bytes(), first.len());
        assert!(statements.next().is_none());
    }

    #[test]
    fn incomplete_statement_keeps_only_complete_prefix_committed() {
        let mut statements = Parser.parse_iter("print(1);sum(\n");
        statements.next().unwrap().unwrap();
        assert_eq!(
            statements.next().unwrap().unwrap_err().status,
            ParseStatus::Incomplete
        );
        assert_eq!(statements.consumed_bytes(), "print(1);".len());
        assert!(statements.next().is_none());
    }

    #[test]
    fn open_input_waits_for_separators_and_partial_tokens() {
        for source in ["1", "# comment", "1e", "0x", "0x1p+", "x<<", "{1"] {
            let mut statements = Parser.parse_iter_open(source);
            assert_eq!(
                statements.next().unwrap().unwrap_err().status,
                ParseStatus::Incomplete,
                "{source:?}"
            );
            assert_eq!(statements.consumed_bytes(), 0);
        }
        for source in ["1e2\n", "0x10;", "0x1p+2\n", "x<<-1\n", "r\"--(v)--\"\n"] {
            let actual = Parser.parse_iter_open(source).next().unwrap().unwrap();
            assert_eq!(
                actual.expressions,
                Parser.parse_all(source).unwrap().expressions
            );
        }
        assert!(Parser.parse_all("1").is_ok());
        assert_eq!(
            Parser.parse_iter_open(" ").next().unwrap().unwrap().status,
            ParseStatus::Null
        );
        assert_eq!(
            Parser
                .parse_iter_open("r\"--")
                .next()
                .unwrap()
                .unwrap_err()
                .status,
            ParseStatus::Error
        );
        assert_eq!(
            Parser.parse_all("1e").unwrap_err().status,
            ParseStatus::Error
        );
        assert_eq!(
            Parser
                .parse_iter_open("@")
                .next()
                .unwrap()
                .unwrap_err()
                .status,
            ParseStatus::Error
        );
    }

    #[test]
    fn open_input_commits_only_terminated_statements_and_their_warnings() {
        let mut statements = Parser.parse_iter_open("1.5L;0x");
        assert_eq!(statements.next().unwrap().unwrap().warnings.len(), 1);
        assert_eq!(statements.consumed_bytes(), "1.5L;".len());
        assert_eq!(
            statements.next().unwrap().unwrap_err().status,
            ParseStatus::Incomplete
        );
        assert_eq!(statements.consumed_bytes(), "1.5L;".len());
    }

    #[test]
    fn statement_iterator_preserves_grouped_and_multiline_syntax() {
        for source in [
            "1 +\n2;3",
            "sum({\nx<-1\nx+2\n},4);5",
            "f<-function(x)\n{x+1};f(2)",
            "cat(\"; # \\\"\"); # comment\n2",
            " # blank input\n",
            "{;;1;;2;}",
            "{if(TRUE)1\nelse 2};3",
        ] {
            let expected = Parser.parse_all(source).unwrap().expressions;
            let mut statements = Parser.parse_iter(source);
            let mut actual = Vec::new();
            for parsed in statements.by_ref() {
                actual.extend(parsed.unwrap().expressions);
            }
            assert_eq!(actual, expected, "{source}");
            assert_eq!(statements.consumed_bytes(), source.len(), "{source}");
        }
    }

    #[test]
    fn else_obeys_newline_and_semicolon_boundaries() {
        for source in [
            ";1",
            "1;;2",
            "1\n;2",
            "if(TRUE)1\nelse 2",
            "if(TRUE)1;else 2",
            "f<-function(x)if(x)1\nelse 2",
            "{if(TRUE)1;else(2)}",
        ] {
            assert!(Parser.parse_all(source).is_err(), "{source}");
        }
        for source in [
            "if(TRUE)1 else 2",
            "{if(TRUE)1\nelse 2}",
            "(if(TRUE)1\nelse 2)",
        ] {
            assert!(Parser.parse_all(source).is_ok(), "{source}");
        }
    }

    #[test]
    fn classifies_unfinished_console_input() {
        for source in [
            "{\n",
            "sum(\n",
            "function(\n",
            "function(x,\n",
            "if(TRUE)\n",
            "for(\n",
            "for(i\n",
            "x$\n",
            "x <-\n",
            "1 +\n",
        ] {
            assert_eq!(
                Parser.parse_all(source).unwrap_err().status,
                ParseStatus::Incomplete,
                "{source}"
            );
        }
        for source in [")", "1 +; 2", "sum(;) ", "function(2)"] {
            assert_eq!(
                Parser.parse_all(source).unwrap_err().status,
                ParseStatus::Error,
                "{source}"
            );
        }
    }

    #[test]
    fn parses_multiline_input_without_merging_statements() {
        for source in [
            "1 +\n2",
            "sum(\n1,\n2\n)",
            "(1\n+ 2)",
            "function(x)\n{\n x +\n 1\n}",
            "sum({\n x <- 1\n x + 2\n})",
        ] {
            assert_eq!(
                Parser.parse_all(source).unwrap().expressions.len(),
                1,
                "{source}"
            );
        }
        for source in ["1\n+2", "x <- 1 # comment\nx", "{1}\n{2}"] {
            assert_eq!(
                Parser.parse_all(source).unwrap().expressions.len(),
                2,
                "{source}"
            );
        }
    }

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
