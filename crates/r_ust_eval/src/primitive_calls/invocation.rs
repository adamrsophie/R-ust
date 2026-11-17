use crate::{CallArgument, EvalError, EvaluatedArgument, RuntimeState, Sexp, SourceFault};
use r_ust_sexp::{canonical_actual_call, PrimitiveId, SexpType};

impl RuntimeState {
    pub(super) fn call_invocation_primitive(
        &mut self,
        id: PrimitiveId,
        arguments: &[EvaluatedArgument],
        call: &Sexp,
    ) -> Result<Sexp, EvalError> {
        let value = one_argument((id.name(), arguments), call)?;
        match id {
            PrimitiveId::Unclass => super::unclass::remove(value),
            PrimitiveId::AsCall => as_call(value, call),
            PrimitiveId::IsPairList => Ok(Sexp::Logical(vec![Some(matches!(
                value.sexptype(),
                SexpType::List | SexpType::Nil
            ))])),
            PrimitiveId::AsEnvironment => self.as_environment(value, call),
            PrimitiveId::IsSymbol => Ok(Sexp::Logical(vec![Some(matches!(
                value,
                Sexp::Symbol(_) | Sexp::MissingValue
            ))])),
            _ => Err(EvalError::Control("invalid invocation primitive".into())),
        }
    }
}

pub(super) fn one_argument<'a>(
    input: (&str, &'a [EvaluatedArgument]),
    call: &Sexp,
) -> Result<&'a Sexp, EvalError> {
    let (name, arguments) = input;
    let [argument] = arguments else {
        return Err(SourceFault::arity(
            format!(
                "{} arguments passed to '{name}' which requires 1",
                arguments.len()
            ),
            call,
        ));
    };
    if let Some(name) = argument
        .name
        .as_deref()
        .filter(|name| !name.is_empty() && *name != "x")
    {
        return Err(SourceFault::arity(
            format!("supplied argument name '{name}' does not match 'x'"),
            call,
        ));
    }
    Ok(&argument.value)
}

fn as_call(value: &Sexp, call: &Sexp) -> Result<Sexp, EvalError> {
    if value.sexptype() == SexpType::Language {
        return canonical_actual_call(value)
            .map_err(|_| EvalError::NotImplemented("as.call internal representation"));
    }
    let projection = crate::ordinary_values::read(value)?;
    let (elements, names_present) = if let Some(view) = projection.as_ref().ordinary_list() {
        (view.elements.to_vec(), view.names_present)
    } else {
        match projection.as_ref() {
            Sexp::Expression {
                elements,
                names_present,
            } => (elements.clone(), *names_present),
            Sexp::PairList(elements) => (elements.as_slice().to_vec(), false),
            Sexp::String(_) | Sexp::AttributedCharacter { .. } => {
                return Err(SourceFault::domain(
                    "as.call(<character>) not feasible; consider str2lang(<char.>)".into(),
                    call,
                ));
            }
            _ => return Err(SourceFault::domain("invalid argument list".into(), call)),
        }
    };
    let mut elements = elements.into_iter();
    let head = elements
        .next()
        .ok_or_else(|| SourceFault::domain("invalid length 0 argument".into(), call))?;
    Ok(Sexp::Call {
        function: Box::new(head.value),
        arguments: elements
            .map(|element| CallArgument {
                name: element
                    .name
                    .or_else(|| names_present.then(|| "NA".into()))
                    .filter(|name| !name.is_empty()),
                value: element.value,
            })
            .collect(),
    })
}
