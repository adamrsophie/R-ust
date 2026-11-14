use r_ust_eval::{Frontend, RuntimeState};
use r_ust_sexp::{AssignmentOperator, Environment, Sexp};

#[path = "dynamic_frame_sdk/callbacks.rs"]
mod callbacks;
#[path = "dynamic_frame_sdk/control.rs"]
mod control;
#[path = "dynamic_frame_sdk/lifetime.rs"]
mod lifetime;

fn bind(runtime: &mut RuntimeState, name: &str, value: Sexp) {
    runtime
        .eval(&Sexp::Assignment {
            operator: AssignmentOperator::Left,
            target: name.into(),
            value: Box::new(value),
        })
        .unwrap();
}

fn last(frontend: &mut Frontend, source: &str) -> Sexp {
    frontend.eval_source(source).unwrap().pop().unwrap()
}

fn global(frontend: &Frontend) -> Environment {
    let Some(Sexp::Environment(frame)) = frontend.runtime().get(".GlobalEnv").unwrap() else {
        panic!("missing runtime global environment")
    };
    frame
}

fn drained(frontend: &mut Frontend) {
    assert_eq!(last(frontend, "sys.nframe()"), Sexp::Integer(vec![Some(0)]));
    assert_eq!(
        last(frontend, "sys.frame(0)"),
        Sexp::Environment(global(frontend))
    );
    assert_eq!(last(frontend, "42L"), Sexp::Integer(vec![Some(42)]));
}

// These are embedding contracts between independent runtimes, not an R cross-session oracle.
#[test]
fn origin_a_closure_observes_dynamic_caller_b() {
    let mut a = Frontend::default();
    let function = last(
        &mut a,
        concat!(
            "a_global<-globalenv();make<-function(){lexical<-environment();",
            "f<-function(caller,top){force(caller);force(top);c(",
            "identical(globalenv(),a_global),",
            "identical(environment(sys.function()),lexical),",
            "identical(parent.frame(),caller),identical(sys.frame(0),top),",
            "identical(sys.function(),f),",
            "identical(parent.env(new.env()),environment()))};f};make()"
        ),
    );
    let a_global = global(&a);
    let mut b = Frontend::default();
    let b_global = global(&b);
    assert_ne!(a_global, b_global);
    bind(b.runtime_mut(), "alias", function);
    bind(b.runtime_mut(), "a_global", Sexp::Environment(a_global));
    bind(b.runtime_mut(), "b_global", Sexp::Environment(b_global));
    assert_eq!(
        last(&mut b, "identical(parent.env(environment(alias)),a_global)"),
        Sexp::Logical(vec![Some(true)])
    );
    for head in ["alias", "(list(alias)[[1]])"] {
        let source = format!("g<-function(){{caller<-environment();{head}(caller,b_global)}};g()");
        assert_eq!(
            last(&mut b, &source),
            Sexp::Logical(vec![Some(true); 6]),
            "{head}"
        );
        drained(&mut b);
    }
    drop(a);
    assert_eq!(last(&mut b, "g()"), Sexp::Logical(vec![Some(true); 6]));
}

#[test]
fn retained_dynamic_frame_survives_after_return_and_origin_drop() {
    lifetime::retained_frame();
}

#[test]
fn host_callback_other_runtime_cannot_rebind_contexts() {
    callbacks::normal_callbacks();
}

#[test]
fn callback_failure_restores_dynamic_contexts() {
    callbacks::failed_callbacks();
}

#[test]
fn equal_environment_foreign_origin_cannot_target_loop() {
    control::equal_environment_origins();
}

#[test]
fn direct_values_apply_and_last_use_actual_activations() {
    let mut a = Frontend::default();
    let function = last(
        &mut a,
        concat!(
            "f<-function(x=0L)c(identical(sys.function(),f),",
            "identical(parent.env(new.env()),environment()),sys.nframe()>0L);f"
        ),
    );
    let mut b = Frontend::default();
    assert_eq!(
        b.runtime_mut()
            .eval(&Sexp::Call {
                function: Box::new(function.clone()),
                arguments: vec![],
            })
            .unwrap(),
        Sexp::Logical(vec![Some(true); 3])
    );
    bind(b.runtime_mut(), "alias", function);
    assert_eq!(
        last(
            &mut b,
            concat!(
                "identical(lapply(1:2,alias),",
                "list(c(TRUE,TRUE,TRUE),c(TRUE,TRUE,TRUE)))"
            )
        ),
        Sexp::Logical(vec![Some(true)])
    );
    let b_global = global(&b);
    bind(a.runtime_mut(), "b_global", Sexp::Environment(b_global));
    let last_function = last(
        &mut a,
        concat!(
            "holder<-new.env();last_fn<-function(){holder$result<-c(",
            "identical(sys.function(),last_fn),identical(parent.frame(),b_global),",
            "identical(sys.frame(0),b_global))};last_fn"
        ),
    );
    let Sexp::Environment(holder) = last(&mut a, "holder") else {
        panic!("missing retained result holder")
    };
    bind(b.runtime_mut(), ".Last", last_function);
    drop(a);
    b.runtime_mut().run_last().unwrap();
    assert_eq!(
        holder.get("result").unwrap(),
        Some(Sexp::Logical(vec![Some(true); 3]))
    );
    drained(&mut b);
}
