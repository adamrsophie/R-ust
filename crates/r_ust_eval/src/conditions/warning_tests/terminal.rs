use super::{EvalError, Row};

pub(super) fn assert_error(error: &EvalError, row: &Row) {
    let original = match error {
        EvalError::Signaled(error) => return assert_error(error, row),
        EvalError::Fresh(fault) => {
            assert_eq!(
                fault.message, row.error_message,
                "{} source payload",
                row.id
            );
            &fault.original
        }
        other => other,
    };
    match row.id {
        "as_environment_top_missing_parent" => assert_eq!(
            original,
            &EvalError::Domain(row.error_message.into()),
            "{}",
            row.id
        ),
        "lapply_special_quote" => assert_eq!(
            original,
            &EvalError::Arity(row.error_message.into()),
            "{}",
            row.id
        ),
        _ => panic!("unreviewed terminal case: {}", row.id),
    }
}
