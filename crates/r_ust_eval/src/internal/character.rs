use crate::{EvalError, Sexp};

pub(super) fn character_values(value: &Sexp) -> Result<Vec<Option<String>>, EvalError> {
    match value {
        Sexp::Managed(_) | Sexp::AttributedAtomic { .. } | Sexp::AttributedAggregate { .. } => {
            let projection = crate::coercion::storage::read(value)?;
            character_values(projection.as_ref())
        }
        Sexp::Factor { codes, .. } => Ok(codes
            .iter()
            .map(|code| code.map(|code| code.to_string()))
            .collect()),
        Sexp::Matrix { data, .. } => character_values(data),
        Sexp::Nil
        | Sexp::MissingArg(_)
        | Sexp::Symbol(_)
        | Sexp::MissingValue
        | Sexp::Expression { .. }
        | Sexp::PairList(_)
        | Sexp::Logical(_)
        | Sexp::Raw(_)
        | Sexp::Integer(_)
        | Sexp::Real(_)
        | Sexp::String(_)
        | Sexp::AttributedCharacter { .. }
        | Sexp::Vector(_)
        | Sexp::NamedList { .. }
        | Sexp::ClassedList { .. }
        | Sexp::List(_)
        | Sexp::DataFrame { .. }
        | Sexp::Dots(_)
        | Sexp::Promise { .. }
        | Sexp::Closure { .. }
        | Sexp::Char(_)
        | Sexp::ListObject(_)
        | Sexp::PairListObject(_)
        | Sexp::Environment(_)
        | Sexp::Bytecode(_)
        | Sexp::Primitive(_)
        | Sexp::InternalCall { .. }
        | Sexp::Function { .. }
        | Sexp::If { .. }
        | Sexp::For { .. }
        | Sexp::While { .. }
        | Sexp::Repeat { .. }
        | Sexp::Break
        | Sexp::Next
        | Sexp::Block(_)
        | Sexp::Subset { .. }
        | Sexp::Subset2D { .. }
        | Sexp::ExtractArguments { .. }
        | Sexp::Extract { .. }
        | Sexp::Dollar { .. }
        | Sexp::Call { .. }
        | Sexp::Assignment { .. }
        | Sexp::SuperAssignment { .. }
        | Sexp::Subassignment { .. }
        | Sexp::MatrixSubassignment { .. }
        | Sexp::ExtractAssignment { .. }
        | Sexp::DollarAssignment { .. }
        | Sexp::MemberAssignment { .. }
        | Sexp::ReplacementAssignment { .. }
        | Sexp::NamesAssignment { .. }
        | Sexp::Binary { .. }
        | Sexp::Unary { .. }
        | Sexp::Parenthesized(_) => crate::string_values(value),
    }
}

#[cfg(test)]
mod managed_tests {
    use super::character_values;
    use crate::{EvalError, Sexp};
    use r_ust_sexp::{AtomicVector, Environment, ListElement, ValueImport, ValueObject};

    #[test]
    fn default_character_coercion_uses_classed_integer_codes() -> Result<(), EvalError> {
        let owner = Environment::global();
        let factor = Sexp::Factor {
            codes: vec![Some(2), None, Some(1)],
            levels: vec!["low".into(), "high".into()],
            ordered: false,
        };
        let attributed = Sexp::AttributedAtomic {
            values: AtomicVector::Integer(vec![Some(2), None, Some(1)]),
            attributes: vec![
                ListElement {
                    name: Some("class".into()),
                    value: Sexp::String(vec![Some("factor".into())]),
                },
                ListElement {
                    name: Some("levels".into()),
                    value: Sexp::String(vec![Some("low".into()), Some("high".into())]),
                },
            ],
        };
        for raw in [factor, attributed] {
            let input = ValueObject::import(&owner, &raw, ValueImport::Evaluated)?.transport()?;
            assert_eq!(
                character_values(&input)?,
                [Some("2".into()), None, Some("1".into())]
            );
        }
        Ok(())
    }
}
