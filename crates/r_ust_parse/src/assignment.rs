use super::{ParseError, SyntaxParser, Token};
use r_ust_sexp::{AssignmentOperator, Sexp};

enum AssignmentKind {
    Local(AssignmentOperator),
    Super,
}

impl SyntaxParser {
    pub(super) fn parse_assignment(&mut self) -> Result<Sexp, ParseError> {
        let left = self.parse_binary(0)?;
        match self.peek() {
            Token::AssignLeft | Token::AssignEqual | Token::AssignLeftSuper => {
                let kind = match self.peek() {
                    Token::AssignLeftSuper => AssignmentKind::Super,
                    Token::AssignEqual => AssignmentKind::Local(AssignmentOperator::Equal),
                    _ => AssignmentKind::Local(AssignmentOperator::Left),
                };
                self.advance();
                let value = self.parse_assignment()?;
                return Self::assignment_from_target(left, value, kind);
            }
            Token::AssignRight | Token::AssignRightSuper => {
                let kind = if matches!(self.peek(), Token::AssignRightSuper) {
                    AssignmentKind::Super
                } else {
                    AssignmentKind::Local(AssignmentOperator::Left)
                };
                self.advance();
                let target = self.parse_assignment()?;
                return Self::assignment_from_target(target, left, kind);
            }
            _ => {}
        }
        Ok(left)
    }

    fn assignment_from_target(
        target_expr: Sexp,
        value: Sexp,
        kind: AssignmentKind,
    ) -> Result<Sexp, ParseError> {
        let operator = match kind {
            AssignmentKind::Local(operator) => operator,
            AssignmentKind::Super => {
                return match target_expr {
                    Sexp::Symbol(target) => Ok(Sexp::SuperAssignment {
                        target,
                        value: Box::new(value),
                    }),
                    _ => Err(ParseError::new(
                        "superassignment currently supports only symbol targets",
                    )),
                }
            }
        };
        match target_expr {
            Sexp::Symbol(target) => Ok(Sexp::Assignment {
                operator,
                target,
                value: Box::new(value),
            }),
            Sexp::Subset {
                value: target,
                index,
            } => match *target {
                Sexp::Symbol(target) => Ok(Sexp::Subassignment {
                    operator,
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
                    operator,
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
                    operator,
                    target,
                    index,
                    value: Box::new(value),
                }),
                other => Self::nested_member_assignment(
                    Sexp::Extract {
                        value: Box::new(other),
                        index,
                    },
                    value,
                    operator,
                ),
            },
            Sexp::Dollar {
                value: target,
                name,
            } => match *target {
                Sexp::Symbol(target) => Ok(Sexp::DollarAssignment {
                    operator,
                    target,
                    name,
                    value: Box::new(value),
                }),
                other => Self::nested_member_assignment(
                    Sexp::Dollar {
                        value: Box::new(other),
                        name,
                    },
                    value,
                    operator,
                ),
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
                            operator,
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

    fn nested_member_assignment(
        target: Sexp,
        value: Sexp,
        operator: AssignmentOperator,
    ) -> Result<Sexp, ParseError> {
        if !member_rooted(&target) {
            return Err(ParseError::new(
                "nested member assignment must target a name",
            ));
        }
        Ok(Sexp::MemberAssignment {
            operator,
            target: Box::new(target),
            value: Box::new(value),
        })
    }
}

fn member_rooted(target: &Sexp) -> bool {
    match target {
        Sexp::Symbol(_) => true,
        Sexp::Dollar { value, .. } | Sexp::Extract { value, .. } => member_rooted(value),
        _ => false,
    }
}
