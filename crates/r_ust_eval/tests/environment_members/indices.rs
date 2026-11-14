use super::*;

#[test]
fn scalar_character_index() {
    let frame = Environment::global().child().unwrap();
    for (name, value) in [("x", 11), ("y", 22), ("NA", 33)] {
        frame.insert(name.into(), &integer(value)).unwrap();
    }
    let mut frontend = Frontend::default();
    bind(
        frontend.runtime_mut(),
        "e",
        Sexp::Environment(frame.clone()),
    );
    frontend.eval_source("count<-0L").unwrap();
    for (index, key) in [
        integer(1),
        Sexp::Logical(vec![Some(true)]),
        Sexp::Nil,
        Sexp::String(vec![]),
        Sexp::String(vec![Some("x".into()), Some("y".into())]),
        Sexp::List(vec![]),
    ]
    .into_iter()
    .enumerate()
    {
        bind(frontend.runtime_mut(), "key", key.clone());
        assert_eq!(
            frontend.eval_source("e[[key]]"),
            Err(EvalError::Type(
                "wrong arguments for subsetting an environment".into()
            )),
            "read {key:?}"
        );
        assert_eq!(
            frontend.eval_source("e[[key]]<-{count<<-count+1L;99L}"),
            Err(EvalError::Type(
                "wrong args for environment subassignment".into()
            )),
            "write {key:?}"
        );
        assert_eq!(
            frontend.runtime().get("count").unwrap(),
            Some(integer(i32::try_from(index + 1).unwrap()))
        );
        assert_eq!(frame.local_len().unwrap(), 3);
        assert_eq!(frame.get_local("x").unwrap(), Some(integer(11)));
        assert_eq!(frame.get_local("y").unwrap(), Some(integer(22)));
        assert_eq!(frame.get_local("NA").unwrap(), Some(integer(33)));
    }
    assert_eq!(
        frontend
            .eval_source("e[[NA_character_]]; e[['NA']]")
            .unwrap(),
        [integer(33), integer(33)]
    );
    frontend.eval_source("e[[NA_character_]]<-44L").unwrap();
    assert_eq!(frame.get_local("NA").unwrap(), Some(integer(44)));
    assert_eq!(frame.local_len().unwrap(), 3);
}

#[test]
fn empty_name_is_rejected_for_both_member_operators() {
    let frame = Environment::global().child().unwrap();
    let mut frontend = Frontend::default();
    bind(
        frontend.runtime_mut(),
        "e",
        Sexp::Environment(frame.clone()),
    );
    for source in ["e[['']]", "e$''", "e[['']]<-1L", "e$''<-1L"] {
        assert_eq!(
            frontend.eval_source(source),
            Err(EvalError::Domain(
                "attempt to use zero-length variable name".into()
            )),
            "{source}"
        );
        assert_eq!(frame.local_len().unwrap(), 0);
    }
}

#[test]
fn member_read_evaluates_object_before_index_validation() {
    let mut frontend = Frontend::default();
    assert_eq!(
        frontend.eval_source("({cat('OBJECT');globalenv()})[[{cat('INDEX');1L}]]"),
        Err(EvalError::Type(
            "wrong arguments for subsetting an environment".into()
        ))
    );
    assert_eq!(
        frontend.runtime_mut().take_output(),
        [
            OutputEvent::Stdout("OBJECT".into()),
            OutputEvent::Stdout("INDEX".into())
        ]
    );
    assert_eq!(frontend.eval_source("7L").unwrap(), [integer(7)]);
}
