use r_ust_sexp::{Sexp, NA_REAL};

use crate::{ParseError, SyntaxParser, Token};

impl SyntaxParser {
    pub(super) fn parse_primary(&mut self) -> Result<Sexp, ParseError> {
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
                "TRUE" => Sexp::Logical(vec![Some(true)]),
                "FALSE" => Sexp::Logical(vec![Some(false)]),
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
}
