use super::{CallArgument, ParseError, Sexp, SyntaxParser, Token};

impl SyntaxParser {
    pub(super) fn parse_call(&mut self) -> Result<Sexp, ParseError> {
        let mut expr = self.parse_primary()?;
        loop {
            match self.peek() {
                Token::LParen => {
                    self.advance();
                    let arguments = self.parse_delimited_arguments(Token::RParen)?;
                    self.expect_token(Token::RParen)?;
                    expr = Sexp::Call {
                        function: Box::new(expr),
                        arguments,
                    };
                }
                Token::LBracket => {
                    self.advance();
                    if matches!(self.peek(), Token::LBracket) {
                        self.advance();
                        let arguments = self.parse_delimited_arguments(Token::RBracket)?;
                        self.expect_token(Token::RBracket)?;
                        self.expect_token(Token::RBracket)?;
                        expr = match arguments.as_slice() {
                            [argument] if argument.name.is_none() => Sexp::Extract {
                                value: Box::new(expr),
                                index: Box::new(argument.value.clone()),
                            },
                            _ => Sexp::ExtractArguments {
                                value: Box::new(expr),
                                arguments,
                            },
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
                        self.expect_token(Token::RBracket)?;
                        expr = Sexp::Subset2D {
                            value: Box::new(expr),
                            row: index,
                            col,
                        };
                        continue;
                    }
                    self.expect_token(Token::RBracket)?;
                    expr = Sexp::Subset {
                        value: Box::new(expr),
                        index,
                    };
                }
                Token::Dollar => {
                    self.advance();
                    let name = self.parse_member()?;
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

    fn parse_delimited_arguments(&mut self, end: Token) -> Result<Vec<CallArgument>, ParseError> {
        self.consume_newlines();
        let mut arguments = Vec::new();
        if *self.peek() == end {
            return Ok(arguments);
        }
        loop {
            let argument = if let Token::Ident(name) | Token::QuotedIdent(name) =
                self.peek().clone()
            {
                if matches!(self.peek_n(1), Some(Token::AssignEqual)) {
                    self.advance();
                    self.advance();
                    self.consume_newlines();
                    let value = if (matches!(self.peek(), Token::Comma) || *self.peek() == end) {
                        Sexp::MissingArg(name.clone())
                    } else {
                        self.parse_expression()?
                    };
                    CallArgument::named(name, value)
                } else {
                    CallArgument::positional(self.parse_expression()?)
                }
            } else if (matches!(self.peek(), Token::Comma) || *self.peek() == end) {
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
}
