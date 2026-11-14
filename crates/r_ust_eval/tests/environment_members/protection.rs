use super::*;
use r_ust_sexp::{EnvironmentRole, EvaluationOrigin, PromiseState};

#[test]
fn protected_writes_preserve_store_and_recover() {
    for root in ["baseenv()", ".BaseNamespaceEnv", "emptyenv()"] {
        let mut frontend = Frontend::default();
        frontend.eval_source(&format!("e<-{root}")).unwrap();
        let Some(Sexp::Environment(frame)) = frontend.runtime().get("e").unwrap() else {
            panic!("protected root")
        };
        let count = frame.local_len().unwrap();
        let identity = frame.get_local("identity").unwrap();
        for (name, member, base_message) in [
            (
                "identity",
                "$identity",
                "cannot change value of locked binding for 'identity'",
            ),
            (
                "identity",
                "[['identity']]",
                "cannot change value of locked binding for 'identity'",
            ),
            (
                "fresh_binding",
                "$fresh_binding",
                "cannot add binding of 'fresh_binding' to the base environment",
            ),
            (
                "fresh_binding",
                "[['fresh_binding']]",
                "cannot add binding of 'fresh_binding' to the base environment",
            ),
        ] {
            let expected = if root == "emptyenv()" {
                "cannot assign values in the empty environment"
            } else {
                base_message
            };
            assert_eq!(
                frontend.eval_source(&format!("e{member}<-{{cat('RHS');1L}}")),
                Err(EvalError::Domain(expected.into())),
                "{root} {name}"
            );
            assert_eq!(
                frontend.runtime_mut().take_output(),
                [OutputEvent::Stdout("RHS".into())]
            );
            assert_eq!(frame.local_len().unwrap(), count);
            assert_eq!(frame.get_local("identity").unwrap(), identity);
            assert_eq!(frame.get_local("fresh_binding").unwrap(), None);
            assert_eq!(frontend.eval_source("7L").unwrap(), [integer(7)]);
        }
    }
}

#[test]
fn index_and_empty_name_errors_precede_protection() {
    for root in ["baseenv()", ".BaseNamespaceEnv", "emptyenv()"] {
        let mut frontend = Frontend::default();
        frontend.eval_source(&format!("e<-{root}")).unwrap();
        let Some(Sexp::Environment(frame)) = frontend.runtime().get("e").unwrap() else {
            panic!("protected root")
        };
        let count = frame.local_len().unwrap();
        for (index, expected) in [
            (
                "1L",
                EvalError::Type("wrong args for environment subassignment".into()),
            ),
            (
                "''",
                EvalError::Domain("attempt to use zero-length variable name".into()),
            ),
        ] {
            assert_eq!(
                frontend.eval_source(&format!("e[[{index}]]<-{{cat('RHS');1L}}")),
                Err(expected)
            );
            assert_eq!(
                frontend.runtime_mut().take_output(),
                [OutputEvent::Stdout("RHS".into())]
            );
            assert_eq!(frame.local_len().unwrap(), count);
        }
    }
}

#[test]
fn lexical_and_member_writes_share_name_aware_protection() {
    let mut frontend = Frontend::default();
    let source = global(frontend.runtime());
    let namespace = source.session_root(EnvironmentRole::BaseNamespace).unwrap();
    let identity = namespace.get_local("identity").unwrap();
    let state = PromiseState::new(
        &expression("identity<-{cat('RHS');1L}"),
        &namespace,
        EvaluationOrigin::Global,
        true,
    )
    .unwrap();
    let frame = source.child().unwrap();
    frame
        .insert("lazy".into(), &Sexp::Promise { state })
        .unwrap();
    bind(frontend.runtime_mut(), "e", Sexp::Environment(frame));
    for source in ["e$lazy", "(function(){identity<<-{cat('RHS');1L}})()"] {
        assert_eq!(
            frontend.eval_source(source),
            Err(EvalError::Domain(
                "cannot change value of locked binding for 'identity'".into()
            ))
        );
        assert_eq!(namespace.get_local("identity").unwrap(), identity);
        assert_eq!(
            frontend.runtime_mut().take_output(),
            [OutputEvent::Stdout("RHS".into())]
        );
        assert_eq!(frontend.eval_source("7L").unwrap(), [integer(7)]);
    }
}
