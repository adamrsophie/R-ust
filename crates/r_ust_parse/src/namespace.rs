use super::{CallArgument, Lexer, ParseError, Sexp, SyntaxParser, Token};

impl Lexer<'_> {
    pub(super) fn colon_token(&mut self) -> Token {
        self.advance();
        if !self.consume(':') {
            Token::Colon
        } else if self.consume(':') {
            Token::NamespaceInternal
        } else {
            Token::NamespaceExport
        }
    }
}

impl SyntaxParser {
    pub(super) fn parse_namespace(&mut self) -> Result<Sexp, ParseError> {
        let package = self.parse_primary()?;
        let name = match self.peek() {
            Token::NamespaceExport => "::",
            Token::NamespaceInternal => ":::",
            _ => return Ok(package),
        };
        if !matches!(package, Sexp::Symbol(_) | Sexp::String(_)) {
            return Err(ParseError::new("unexpected namespace selector"));
        }
        self.advance();
        self.consume_newlines();
        let member = match self.advance().clone() {
            Token::Ident(name) | Token::QuotedIdent(name) => Sexp::Symbol(name),
            Token::String(value) => Sexp::String(vec![Some(value)]),
            Token::Eof => return Err(ParseError::incomplete("unexpected end of input")),
            _ => return Err(ParseError::new("unexpected namespace name")),
        };
        Ok(Sexp::Call {
            function: Box::new(Sexp::Symbol(name.into())),
            arguments: vec![
                CallArgument::positional(package),
                CallArgument::positional(member),
            ],
        })
    }
}
