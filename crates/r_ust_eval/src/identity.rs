use super::*;

pub(super) fn sexp_identical(left: &Sexp, right: &Sexp) -> bool {
    match (left, right) {
        (Sexp::Nil, Sexp::Nil) | (Sexp::MissingValue, Sexp::MissingValue) => true,
        (Sexp::PairList(left), Sexp::PairList(right)) => {
            list_elements_identical(left.as_slice(), right.as_slice())
        }
        (
            Sexp::Call {
                function: left_head,
                arguments: left,
            },
            Sexp::Call {
                function: right_head,
                arguments: right,
            },
        ) => {
            sexp_identical(left_head, right_head)
                && left.len() == right.len()
                && left.iter().zip(right).all(|(left, right)| {
                    left.name == right.name && sexp_identical(&left.value, &right.value)
                })
        }
        (Sexp::Primitive(left), Sexp::Primitive(right)) => left == right,
        (Sexp::MissingArg(left), Sexp::MissingArg(right)) => left == right,
        (Sexp::Symbol(left), Sexp::Symbol(right)) => left == right,
        (Sexp::Logical(left), Sexp::Logical(right)) => left == right,
        (Sexp::Integer(left), Sexp::Integer(right)) => left == right,
        (Sexp::Real(left), Sexp::Real(right)) => real_values_identical(left, right),
        (Sexp::String(left), Sexp::String(right)) => left == right,
        (Sexp::Vector(left), Sexp::Vector(right)) => {
            left.len() == right.len()
                && left
                    .iter()
                    .zip(right)
                    .all(|(left, right)| sexp_identical(left, right))
        }
        (Sexp::List(left), Sexp::List(right)) | (Sexp::Dots(left), Sexp::Dots(right)) => {
            list_elements_identical(left, right)
        }
        (
            Sexp::Factor {
                codes: left_codes,
                levels: left_levels,
                ordered: left_ordered,
            },
            Sexp::Factor {
                codes: right_codes,
                levels: right_levels,
                ordered: right_ordered,
            },
        ) => {
            left_codes == right_codes
                && left_levels == right_levels
                && left_ordered == right_ordered
        }
        (
            Sexp::DataFrame {
                columns: left_columns,
                nrow: left_nrow,
            },
            Sexp::DataFrame {
                columns: right_columns,
                nrow: right_nrow,
            },
        ) => left_nrow == right_nrow && list_elements_identical(left_columns, right_columns),
        (
            Sexp::Matrix {
                data: left_data,
                nrow: left_nrow,
                ncol: left_ncol,
            },
            Sexp::Matrix {
                data: right_data,
                nrow: right_nrow,
                ncol: right_ncol,
            },
        ) => {
            left_nrow == right_nrow
                && left_ncol == right_ncol
                && sexp_identical(left_data, right_data)
        }
        (Sexp::Promise { state: left }, Sexp::Promise { state: right }) => left == right,
        (
            Sexp::Closure {
                parameters: left_parameters,
                body: left_body,
                environment: left_environment,
            },
            Sexp::Closure {
                parameters: right_parameters,
                body: right_body,
                environment: right_environment,
            },
        ) => {
            left_environment == right_environment
                && left_parameters == right_parameters
                && sexp_identical(left_body, right_body)
        }
        (
            Sexp::Function {
                parameters: left_parameters,
                body: left_body,
            },
            Sexp::Function {
                parameters: right_parameters,
                body: right_body,
            },
        ) => left_parameters == right_parameters && sexp_identical(left_body, right_body),
        _ => left == right,
    }
}

pub(super) fn real_values_identical(left: &[f64], right: &[f64]) -> bool {
    left.len() == right.len()
        && left.iter().zip(right).all(|(left, right)| {
            left == right
                || (left.is_nan() && right.is_nan() && is_na_real(*left) == is_na_real(*right))
        })
}

pub(super) fn list_elements_identical(left: &[ListElement], right: &[ListElement]) -> bool {
    left.len() == right.len()
        && left.iter().zip(right).all(|(left, right)| {
            left.name == right.name && sexp_identical(&left.value, &right.value)
        })
}
