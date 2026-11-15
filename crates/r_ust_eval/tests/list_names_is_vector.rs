use r_ust_eval::{Frontend, RuntimeState};
use r_ust_sexp::{AssignmentOperator, Environment, ListElement, ListObject, Sexp};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn named(length: usize, class: Option<Sexp>) -> Sexp {
    Sexp::NamedList {
        elements: (0..length)
            .map(|_| ListElement {
                name: None,
                value: Sexp::Integer(vec![Some(7)]),
            })
            .collect(),
        class: class.map(Box::new),
        class_first: false,
    }
}

fn bind(runtime: &mut RuntimeState, value: Sexp) -> Result {
    runtime.eval(&Sexp::Assignment {
        operator: AssignmentOperator::Left,
        target: "x".into(),
        value: Box::new(value),
    })?;
    Ok(())
}

fn check(value: Sexp, eligible: bool) -> Result {
    let mut frontend = Frontend::default();
    bind(frontend.runtime_mut(), value.clone())?;
    let result = frontend.eval_source("c(is.vector(x),is.vector(x,'any'),is.vector(x,'list'),is.vector(x,'expression'),is.vector(x,'numeric'),is.vector(x,'pairlist'),is.vector(x,'character'))")?;
    assert_eq!(
        result.last(),
        Some(&Sexp::Logical(vec![
            Some(eligible),
            Some(eligible),
            Some(eligible),
            Some(false),
            Some(false),
            Some(false),
            Some(false)
        ]))
    );
    assert_eq!(frontend.runtime().get("x")?, Some(value));
    Ok(())
}

#[test]
fn present_empty_and_all_na_names_allow_any_and_exact_list_modes() -> Result {
    for length in [0, 1, 2] {
        check(named(length, None), true)?;
    }
    Ok(())
}

#[test]
fn class_attributes_reject_every_mode_even_with_empty_or_na_names() -> Result {
    for length in [0, 1, 2] {
        check(
            named(length, Some(Sexp::String(vec![Some("box".into())]))),
            false,
        )?;
    }
    Ok(())
}

#[test]
fn explicit_named_variant_with_mixed_names_uses_same_attribute_rule() -> Result {
    let value = Sexp::NamedList {
        elements: vec![
            ListElement {
                name: None,
                value: Sexp::Nil,
            },
            ListElement {
                name: Some(String::new()),
                value: Sexp::Nil,
            },
            ListElement {
                name: Some("NA".into()),
                value: Sexp::Nil,
            },
        ],
        class: None,
        class_first: false,
    };
    check(value, true)
}

#[test]
fn graph_projection_preserves_names_only_eligibility_and_class_rejection() -> Result {
    let owner = Environment::global();
    for length in [0, 2] {
        let value = named(length, None);
        check(
            Sexp::ListObject(ListObject::from_value(&owner, &value)?),
            true,
        )?;
        let value = named(length, Some(Sexp::String(vec![Some("box".into())])));
        check(
            Sexp::ListObject(ListObject::from_value(&owner, &value)?),
            false,
        )?;
    }
    Ok(())
}

#[test]
fn caller_matching_reorders_names_and_accepts_partial_mode_once() -> Result {
    let mut frontend = Frontend::default();
    bind(frontend.runtime_mut(), named(2, None))?;
    let values=frontend.eval_source("hits<-0L;f<-function(){hits<<-hits+1L;x};a<-is.vector(mode='list',x=f());b<-is.vector(m='any',x=x);c(a,b,hits==1L,identical(names(x),c(NA_character_,NA_character_)))")?;
    assert_eq!(values.last(), Some(&Sexp::Logical(vec![Some(true); 4])));
    Ok(())
}

#[test]
fn caller_matching_errors_retain_the_actual_outer_call() -> Result {
    let mut frontend = Frontend::default();
    bind(frontend.runtime_mut(), named(0, None))?;
    for source in [
        "is.vector(x=x,x=x)",
        "is.vector(mode='list',mode='any',x=x)",
        "is.vector(x,1L)",
        "is.vector(x,'list',extra=1L)",
    ] {
        let assertion=format!("tryCatch({{ {source};FALSE }},error=function(e) identical(conditionCall(e),quote({source})))");
        assert_eq!(
            frontend.eval_source(&assertion)?.last(),
            Some(&Sexp::Logical(vec![Some(true)])),
            "{source}"
        );
    }
    Ok(())
}
