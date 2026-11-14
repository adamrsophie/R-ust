use crate::test_support::{fixture_json, fixture_node};
mod sdk;
use super::{format_language, LanguagePrintError};
use crate::{CallArgument, Sexp};

#[test]
fn canonical_ascii_tree_matches_frozen_r_print_bytes() -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(
        check_fixture(
            include_str!("../../../tests/fixtures/language_printer/language-printer-r-4.6.0.tsv"),
            include_str!("../../../tests/fixtures/language_printer/outcomes.tsv")
        )?,
        (1401, 111, 27)
    );
    Ok(())
}

#[test]
fn inlist_context_matches_r_replacement_and_prefix_calls() -> Result<(), Box<dyn std::error::Error>>
{
    assert_eq!(
        check_fixture(
            include_str!("../../../tests/fixtures/language_printer/inlist-r-4.6.0.tsv"),
            ""
        )?,
        (10, 0, 0)
    );
    Ok(())
}

#[test]
fn original_quote_language_sections_match_independent_typed_r_trees(
) -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(
        check_fixture(
            include_str!("../../../tests/fixtures/language_printer/frozen45-r-4.6.0.tsv"),
            ""
        )?,
        (45, 0, 0)
    );
    Ok(())
}

#[test]
fn namespace_arity_and_computed_operands_match_the_pinned_r_branch(
) -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(
        check_fixture(
            include_str!("../../../tests/fixtures/language_printer/namespace-r-4.6.0.tsv"),
            ""
        )?,
        (8, 0, 0)
    );
    Ok(())
}

#[test]
fn return_and_foreign_computed_heads_match_r_caller_parentheses(
) -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(
        check_fixture(
            include_str!("../../../tests/fixtures/language_printer/computed-head-r-4.6.0.tsv"),
            ""
        )?,
        (6, 0, 0)
    );
    Ok(())
}

#[test]
fn integer_na_uses_the_canonical_option_payload() {
    for (value, expected) in [
        (Some(i32::MIN), Err(LanguagePrintError::UnsupportedNode)),
        (None, Ok("NA_integer_".into())),
    ] {
        assert_eq!(format_language(&Sexp::Integer(vec![value])), expected);
    }
}

fn check_fixture(
    fixture: &str,
    outcomes: &str,
) -> Result<(usize, usize, usize), Box<dyn std::error::Error>> {
    let mut expected = std::collections::HashMap::new();
    for line in outcomes.lines().skip(1) {
        let (id, outcome) = line.split_once('\t').ok_or("outcome columns")?;
        assert!(
            expected.insert(id, outcome).is_none(),
            "duplicate outcome {id}"
        );
    }
    let mut accepted = 0;
    let mut malformed = 0;
    let mut rejected = 0;
    let mut mismatches = Vec::new();
    for line in fixture.lines().skip(1) {
        let cells: Vec<_> = line.split('\t').collect();
        let [id, _, node, output] = cells.as_slice() else {
            return Err("fixture columns".into());
        };
        let outcome = if outcomes.is_empty() {
            "accepted"
        } else {
            expected.remove(id).ok_or("missing expected outcome")?
        };
        let json = String::from_utf8(fixture_json::hex(node)?)?;
        let decoded = fixture_node::node(&fixture_json::parse(&json)?);
        match decoded {
            Ok(value) => match format_language(&value) {
                Ok(actual) if outcome == "accepted" => {
                    let expected = fixture_json::hex(output)?;
                    if actual.as_bytes() != expected {
                        mismatches.push(format!(
                            "{id}: {json}\nactual {actual:?}\nexpected {:?}",
                            String::from_utf8(expected)?
                        ));
                    }
                    accepted += 1;
                }
                Err(LanguagePrintError::UnsupportedText) if outcome == "text" => rejected += 1,
                Err(LanguagePrintError::MalformedCall) if outcome == "malformed" => malformed += 1,
                result => panic!("{id}: expected {outcome}, actual {result:?}: {json}"),
            },
            Err(error) if error == "unsupported encoded bytes" && outcome == "text" => {
                rejected += 1
            }
            Err(error) => return Err(format!("{id}: {error}").into()),
        }
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
    assert!(expected.is_empty(), "unused outcome rows");
    Ok((accepted, malformed, rejected))
}

#[test]
fn malformed_runtime_and_nonascii_nodes_are_checked() {
    let call = |name: &str, args: Vec<Sexp>| Sexp::Call {
        function: Box::new(Sexp::Symbol(name.into())),
        arguments: args.into_iter().map(CallArgument::positional).collect(),
    };
    for value in [
        call("if", vec![]),
        call("function", vec![Sexp::Nil]),
        call("!", vec![]),
        call("f", vec![Sexp::Integer(vec![])]),
        call("f", vec![Sexp::Symbol("é".into())]),
        call("f", vec![Sexp::String(vec![Some("\0".into())])]),
        Sexp::Break,
    ] {
        assert!(format_language(&value).is_err(), "{value:?}");
    }
}

#[test]
fn fixture_transport_rejects_malformed_json_and_nodes() {
    for value in ["", "[", "[\"nil\"]x", "[\"nil\",]", "[\"a\\n\"]"] {
        assert!(fixture_json::parse(value).is_err(), "{value}");
    }
    for value in [
        "[\"unknown\"]",
        "[\"real\",\"1\"]",
        "[\"nil\",\"x\"]",
        "[\"symbol\",\"0\"]",
        "[\"symbol\",\"gg\"]",
        "[\"string\",\"61\",\"bogus\"]",
    ] {
        assert!(
            fixture_node::node(&fixture_json::parse(value).unwrap()).is_err(),
            "{value}"
        );
    }
}
