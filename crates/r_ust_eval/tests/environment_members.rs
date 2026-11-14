use r_ust_eval::{EvalError, Frontend, OutputEvent, RuntimeState};
use r_ust_parse::Parser;
use r_ust_sexp::{AssignmentOperator, Environment, Sexp};

#[path = "environment_members/assignment.rs"]
mod assignment;
#[path = "environment_members/indices.rs"]
mod indices;
#[path = "environment_members/language.rs"]
mod language;
#[path = "environment_members/promises.rs"]
mod promises;
#[path = "environment_members/protection.rs"]
mod protection;

fn integer(value: i32) -> Sexp {
    Sexp::Integer(vec![Some(value)])
}

fn expression(source: &str) -> Sexp {
    Parser.parse_all(source).unwrap().expressions.remove(0)
}

fn global(runtime: &RuntimeState) -> Environment {
    let Some(Sexp::Environment(frame)) = runtime.get(".GlobalEnv").unwrap() else {
        panic!("global environment")
    };
    frame
}

fn bind(runtime: &mut RuntimeState, name: &str, value: Sexp) {
    runtime
        .eval(&Sexp::Assignment {
            operator: AssignmentOperator::Left,
            target: name.into(),
            value: Box::new(value),
        })
        .unwrap();
}

#[test]
fn alias_and_null() {
    let parent = Environment::global();
    parent.insert("parent_only".into(), &integer(8)).unwrap();
    let child = parent.child().unwrap();
    child.insert("alpha".into(), &integer(1)).unwrap();
    let mut frontend = Frontend::default();
    bind(
        frontend.runtime_mut(),
        "e",
        Sexp::Environment(child.clone()),
    );
    frontend.eval_source("alias<-e; holder<-list(e)").unwrap();

    assert_eq!(
        frontend
            .eval_source("e$parent_only; e[['parent_only']]; e$al; e[['absent']]")
            .unwrap(),
        [Sexp::Nil, Sexp::Nil, Sexp::Nil, Sexp::Nil]
    );
    assert_eq!(
        frontend
            .eval_source("e$alpha<-2L; alias[['alpha']]; holder[[1L]]$alpha")
            .unwrap(),
        [integer(2), integer(2), integer(2)]
    );
    frontend
        .eval_source("e[['alpha']]<-NULL; e$new<-NULL")
        .unwrap();
    assert_eq!(child.get_local("alpha").unwrap(), Some(Sexp::Nil));
    assert_eq!(child.get_local("new").unwrap(), Some(Sexp::Nil));
    assert_eq!(child.get_local("absent").unwrap(), None);
    assert_eq!(child.local_len().unwrap(), 2);
    assert_eq!(parent.get_local("parent_only").unwrap(), Some(integer(8)));
}

#[test]
fn raw_missing_member_is_a_value() {
    let child = Environment::global().child().unwrap();
    child
        .insert("raw".into(), &Sexp::MissingArg("raw".into()))
        .unwrap();
    child
        .insert("computed".into(), &Sexp::MissingValue)
        .unwrap();
    let mut frontend = Frontend::default();
    bind(frontend.runtime_mut(), "e", Sexp::Environment(child));

    assert_eq!(
        frontend
            .eval_source("e$raw; e[['raw']]; e$computed; e[['computed']]")
            .unwrap(),
        [
            Sexp::MissingValue,
            Sexp::MissingValue,
            Sexp::MissingValue,
            Sexp::MissingValue
        ]
    );
}

#[test]
fn nested_environment_writeback() {
    let mut frontend = Frontend::default();
    assert_eq!(
        frontend
            .eval_source("e<-globalenv(); h<-list(e); h[[1L]]$x<-9L; e$x")
            .unwrap()
            .last(),
        Some(&integer(9))
    );
    frontend
        .eval_source("h[[{cat('OUTER\\n');1L}]][[{cat('INNER\\n');'x'}]]<-{cat('VALUE\\n');3L}")
        .unwrap();
    assert_eq!(
        frontend.runtime_mut().take_output(),
        ["VALUE\n", "OUTER\n", "INNER\n", "OUTER\n"].map(|text| OutputEvent::Stdout(text.into()))
    );
    assert_eq!(frontend.eval_source("e$x").unwrap(), [integer(3)]);
}
