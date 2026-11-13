use super::{
    data_args, flatten_numeric_data_args, integer_arithmetic_operand, is_na_real, numeric_result,
    preserve_na, real_to_integer, EvalError, EvaluatedArgument, Sexp,
};

pub(super) fn sum(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let values = flatten_numeric_data_args(args)?;
    let value = preserve_na(values.iter().fold(0.0, |sum, value| sum + value), &values);
    let integer = data_args(args, &["na.rm"])
        .iter()
        .all(|arg| integer_arithmetic_operand(&arg.value))
        && (is_na_real(value) || real_to_integer(value).is_some());
    Ok(numeric_result(vec![value], integer))
}

pub(super) fn product(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let values = flatten_numeric_data_args(args)?;
    Ok(Sexp::Real(vec![preserve_na(
        values.iter().product(),
        &values,
    )]))
}
