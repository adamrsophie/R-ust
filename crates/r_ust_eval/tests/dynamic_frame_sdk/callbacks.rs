use super::{bind, drained, global, last};
use r_ust_eval::{EvalError, Frontend, OutputEvent};
use r_ust_sexp::{Environment, Sexp};
use std::cell::RefCell;

#[derive(Clone, Copy, PartialEq)]
enum Channel {
    Output,
    Input,
}
struct Probe {
    b_global: Environment,
    fail: Option<Channel>,
    output_calls: usize,
    input_calls: usize,
}
thread_local! {
    static PROBE: RefCell<Option<Probe>> = const { RefCell::new(None) };
}
struct ProbeGuard;
impl Drop for ProbeGuard {
    fn drop(&mut self) {
        PROBE.with(|probe| *probe.borrow_mut() = None);
    }
}

fn install(b: &mut Frontend, fail: Option<Channel>) -> ProbeGuard {
    let b_global = global(b);
    bind(
        b.runtime_mut(),
        "b_global",
        Sexp::Environment(b_global.clone()),
    );
    PROBE.with(|probe| {
        assert!(probe.borrow().is_none());
        *probe.borrow_mut() = Some(Probe {
            b_global,
            fail,
            output_calls: 0,
            input_calls: 0,
        });
    });
    b.runtime_mut().set_session_context(Vec::new(), true);
    b.runtime_mut().set_output_sink(output);
    b.runtime_mut().set_input_reader(input);
    ProbeGuard
}

fn other_runtime(channel: Channel) -> std::io::Result<()> {
    let mut c = Frontend::default();
    let c_global = global(&c);
    let fail = PROBE.with(|probe| {
        let mut borrowed = probe.borrow_mut();
        let state = borrowed.as_mut().unwrap();
        assert_ne!(c_global, state.b_global);
        match channel {
            Channel::Output => state.output_calls += 1,
            Channel::Input => state.input_calls += 1,
        }
        state.fail == Some(channel)
    });
    assert_eq!(last(&mut c, "sys.nframe()"), Sexp::Integer(vec![Some(0)]));
    assert_eq!(
        last(&mut c, "sys.frame(0)"),
        Sexp::Environment(c_global.clone())
    );
    bind(c.runtime_mut(), "c_global", Sexp::Environment(c_global));
    assert_eq!(
        last(
            &mut c,
            concat!(
                "(function()c(sys.nframe()==1L,identical(parent.frame(),c_global),",
                "identical(sys.frame(0),c_global)))()"
            )
        ),
        Sexp::Logical(vec![Some(true); 3])
    );
    drained(&mut c);
    if fail {
        Err(std::io::Error::other("callback failed after C"))
    } else {
        Ok(())
    }
}

fn output(_: OutputEvent) -> std::io::Result<()> {
    other_runtime(Channel::Output)
}
fn input() -> std::io::Result<String> {
    other_runtime(Channel::Input)?;
    Ok("C".into())
}

pub fn normal_callbacks() {
    let mut b = Frontend::default();
    let guard = install(&mut b, None);
    assert_eq!(
        last(
            &mut b,
            concat!(
                "f<-function(){before<-environment();depth<-sys.nframe();",
                "cat('tick');answer<-readline('');c(identical(environment(),before),",
                "identical(sys.frame(0),b_global),sys.nframe()==depth,",
                "identical(sys.function(),f),identical(answer,'C'))};f()"
            )
        ),
        Sexp::Logical(vec![Some(true); 5])
    );
    PROBE.with(|probe| {
        let borrowed = probe.borrow();
        let state = borrowed.as_ref().unwrap();
        assert!(state.output_calls >= 1);
        assert_eq!(state.input_calls, 1);
    });
    drained(&mut b);
    drop(guard);
    PROBE.with(|probe| assert!(probe.borrow().is_none()));
}

pub fn failed_callbacks() {
    for channel in [Channel::Output, Channel::Input] {
        let mut b = Frontend::default();
        let guard = install(&mut b, Some(channel));
        let source = match channel {
            Channel::Output => "(function(){before<<-environment();cat('tick')})()",
            Channel::Input => "(function(){before<<-environment();readline('')})()",
        };
        let error = b.eval_source(source).unwrap_err();
        match (channel, error) {
            (Channel::Output, EvalError::Output(message))
            | (Channel::Input, EvalError::Input(message)) => {
                assert_eq!(message, "callback failed after C")
            }
            other => panic!("wrong callback failure: {:?}", other.1),
        }
        assert_eq!(
            last(&mut b, "identical(parent.env(before),b_global)"),
            Sexp::Logical(vec![Some(true)])
        );
        drained(&mut b);
        drop(guard);
        PROBE.with(|probe| assert!(probe.borrow().is_none()));
    }
}
