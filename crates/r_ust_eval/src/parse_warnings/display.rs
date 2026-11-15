use super::WarningRecord;
use crate::{EvalError, RuntimeState};

pub(super) fn immediate(
    runtime: &mut RuntimeState,
    warning: &WarningRecord,
    no_break: bool,
) -> Result<String, EvalError> {
    let Some(call) = &warning.call else {
        return Ok(format!("Warning: {}\n", warning.message));
    };
    let call = runtime.deparse_call_line(call)?;
    let first = warning.message.split('\n').next().unwrap_or("");
    let separator = if !no_break && 18 + call.len() + first.len() > 75 {
        "\n  "
    } else {
        " "
    };
    Ok(format!(
        "Warning in {call} :{separator}{}\n",
        warning.message
    ))
}

pub(super) fn deferred(
    runtime: &mut RuntimeState,
    warnings: &[WarningRecord],
) -> Result<String, EvalError> {
    let mut text = if warnings.len() == 1 {
        "Warning message:\n"
    } else {
        "Warning messages:\n"
    }
    .to_owned();
    for (index, warning) in warnings.iter().enumerate() {
        let multiple = warnings.len() > 1;
        if multiple {
            text.push_str(&format!("{}: ", index + 1));
        }
        match &warning.call {
            None => text.push_str(&format!("{} \n", warning.message)),
            Some(call) => {
                let call = runtime.deparse_call_line(call)?;
                let first = warning.message.split('\n').next().unwrap_or("");
                let overhead = if multiple { 10 } else { 6 };
                let separator = if overhead + call.len() + first.len() > 75 {
                    "\n  "
                } else {
                    " "
                };
                text.push_str(&format!("In {call} :{separator}{}\n", warning.message));
            }
        }
    }
    Ok(text)
}
