use super::*;
use r_ust_sexp::{canonical_language, LanguageError};

fn canonical(source: &str) -> Result<Sexp, Box<dyn std::error::Error>> {
    let parsed = Parser.parse_all(source).map_err(|error| error.message)?;
    let [expression] = parsed.expressions.as_slice() else {
        return Err(format!("expected one expression: {source}").into());
    };
    Ok(canonical_language(expression)?)
}

fn metadata(value: &Sexp, kind: &str, class: &str, length: &str, id: &str) {
    assert_eq!(typeof_value(value), kind, "{id} typeof");
    assert_eq!(
        class_value(value),
        Sexp::String(vec![Some(class.into())]),
        "{id} class"
    );
    assert_eq!(value_len(value).to_string(), length, "{id} length");
}

fn rows(source: &str) -> impl Iterator<Item = Vec<&str>> {
    source
        .lines()
        .skip(1)
        .map(|line| line.split('\t').collect())
}

const METADATA: &str = include_str!("../tests/fixtures/canonical_language/metadata.tsv");

#[test]
fn canonical_language_pairlists_are_lists_but_calls_and_empty_formals_are_not(
) -> Result<(), Box<dyn std::error::Error>> {
    for (value, expected) in [
        (
            Sexp::pairlist(vec![ListElement {
                name: Some("x".into()),
                value: Sexp::MissingValue,
            }]),
            true,
        ),
        (Sexp::pairlist(vec![]), false),
        (canonical("f(x)")?, false),
    ] {
        let data = Sexp::Extract {
            value: Box::new(Sexp::List(vec![ListElement { name: None, value }])),
            index: Box::new(Sexp::Integer(vec![Some(1)])),
        };
        let result = RuntimeState::default().eval(&Sexp::Call {
            function: Box::new(Sexp::Symbol("is.list".into())),
            arguments: vec![CallArgument::positional(data)],
        })?;
        assert_eq!(result, Sexp::Logical(vec![Some(expected)]));
    }
    Ok(())
}

#[test]
fn canonical_language_metadata_matches_r() -> Result<(), Box<dyn std::error::Error>> {
    for row in rows(METADATA) {
        let [id, source, kind, class, length] = row.as_slice() else {
            return Err("metadata row".into());
        };
        metadata(&canonical(source)?, kind, class, length, id);
    }
    for row in rows(include_str!(
        "../tests/fixtures/canonical_language/modes.tsv"
    )) {
        let [id, source, path, mode] = row.as_slice() else {
            return Err("mode row".into());
        };
        let value = canonical(source)?;
        let value = if *path == "-" {
            &value
        } else {
            projection(&value, path)?
        };
        assert_eq!(mode_value(value), *mode, "{id} mode");
    }
    Ok(())
}

fn projection<'a>(mut value: &'a Sexp, path: &str) -> Result<&'a Sexp, Box<dyn std::error::Error>> {
    for position in path.split('/') {
        let position = position
            .parse::<usize>()?
            .checked_sub(1)
            .ok_or("zero position")?;
        value = match value {
            Sexp::Call { function, .. } if position == 0 => function,
            Sexp::Call { arguments, .. } => {
                &arguments.get(position - 1).ok_or("call position")?.value
            }
            Sexp::PairList(elements) => {
                &elements
                    .as_slice()
                    .get(position)
                    .ok_or("pairlist position")?
                    .value
            }
            _ => return Err(format!("invalid projection {path}: {value:?}").into()),
        };
    }
    Ok(value)
}

#[test]
fn canonical_language_formals_holes_and_source_slots_match_r(
) -> Result<(), Box<dyn std::error::Error>> {
    for row in rows(include_str!(
        "../tests/fixtures/canonical_language/projections.tsv"
    )) {
        let [id, path, kind, class, length, null, missing] = row.as_slice() else {
            return Err("projection row".into());
        };
        let source = rows(METADATA)
            .find(|row| row.first() == Some(id))
            .ok_or("projection source")?;
        let value = canonical(source[1])?;
        let value = projection(&value, path)?;
        metadata(value, kind, class, length, id);
        assert_eq!(
            matches!(value, Sexp::Nil),
            *null == "TRUE",
            "{id} {path} NULL"
        );
        assert_eq!(
            matches!(value, Sexp::MissingValue),
            *missing == "TRUE",
            "{id} {path} missing"
        );
    }
    Ok(())
}

#[test]
fn canonical_language_identity_matches_r() -> Result<(), Box<dyn std::error::Error>> {
    for table in [
        include_str!("../tests/fixtures/canonical_language/equalities.tsv"),
        include_str!("../tests/fixtures/canonical_language/assignment-equalities.tsv"),
        include_str!("../tests/fixtures/canonical_language/nested-identity-equalities.tsv"),
        include_str!("../tests/fixtures/canonical_language/replacement-equalities.tsv"),
    ] {
        for row in rows(table) {
            let [id, left, right, identical] = row.as_slice() else {
                return Err("identity row".into());
            };
            assert_eq!(
                sexp_identical(&canonical(left)?, &canonical(right)?),
                *identical == "TRUE",
                "{id}"
            );
        }
    }
    Ok(())
}

#[test]
fn canonical_language_dollar_runtime_matches_r() -> Result<(), Box<dyn std::error::Error>> {
    for row in rows(include_str!(
        "../tests/fixtures/canonical_language/runtime.tsv"
    )) {
        let [id, source, kind, class, length, integer] = row.as_slice() else {
            return Err("runtime row".into());
        };
        let values = Frontend::default().eval_source(source)?;
        let value = values.last().ok_or("runtime result")?;
        metadata(value, kind, class, length, id);
        assert_eq!(value, &Sexp::Integer(vec![Some(integer.parse()?)]), "{id}");
    }
    for source in [
        "function(\"x\") x",
        "function(x) x",
        "1L -> x; x",
        "x=1L; x",
    ] {
        if source == "function(\"x\") x" {
            assert!(Parser.parse_all(source).is_err());
        } else {
            Frontend::default().eval_source(source)?;
        }
    }
    Ok(())
}

#[test]
fn canonical_language_boundary_rejects_runtime_handles_and_internal_dispatch(
) -> Result<(), Box<dyn std::error::Error>> {
    let internal = Sexp::InternalCall {
        operation: r_ust_sexp::InternalOperation::Vector,
        arguments: vec![],
    };
    let nested = Sexp::Call {
        function: Box::new(Sexp::Symbol("f".into())),
        arguments: vec![CallArgument::positional(internal)],
    };
    assert_eq!(
        canonical_language(&nested),
        Err(LanguageError::InternalCall)
    );
    let frame = Environment::global();
    let state = r_ust_sexp::PromiseState::new(
        &Sexp::Nil,
        &frame,
        r_ust_sexp::EvaluationOrigin::Global,
        true,
    )?;
    for value in [
        Sexp::Promise { state },
        Sexp::Closure {
            parameters: vec![],
            body: Box::new(Sexp::Nil),
            environment: ClosureEnvironment::Captured(frame),
        },
        Sexp::List(vec![]),
    ] {
        assert_eq!(canonical_language(&value), Err(LanguageError::RuntimeValue));
    }
    for source in [
        "{side <- side+1L; stop(\"FORCED\")}",
        "unresolved",
        "return(x)",
        "break",
        "next",
    ] {
        let value = canonical(source)?;
        assert!(matches!(value, Sexp::Call { .. } | Sexp::Symbol(_)));
    }
    let missing = canonical_language(&Sexp::MissingArg("x".into()))?;
    assert!(sexp_identical(
        &missing,
        &canonical_language(&Sexp::MissingArg("y".into()))?
    ));
    metadata(&missing, "symbol", "name", "1", "missing");
    assert!(!sexp_identical(&missing, &Sexp::Nil));
    assert!(!sexp_identical(
        &Sexp::MissingArg("x".into()),
        &Sexp::MissingArg("y".into())
    ));
    assert_eq!(Sexp::pairlist(vec![]), Sexp::Nil);
    let value = canonical("function(x, y = f(x+1L)) y")?;
    assert!(sexp_identical(&value, &canonical_language(&value)?));
    Ok(())
}

#[test]
fn canonical_language_calls_are_rejected_as_vector_lengths(
) -> Result<(), Box<dyn std::error::Error>> {
    for source in ["f()", "f(x, y)"] {
        assert_eq!(
            RuntimeState::default().vector_length(&canonical(source)?),
            Err(EvalError::Domain("invalid 'length' argument".into()))
        );
    }
    Ok(())
}
