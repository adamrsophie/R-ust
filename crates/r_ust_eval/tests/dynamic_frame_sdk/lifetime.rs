use super::{bind, drained, global, last};
use r_ust_eval::Frontend;
use r_ust_sexp::Sexp;

pub fn retained_frame() {
    for iteration in 0..64 {
        let mut a = Frontend::default();
        let function = last(&mut a, "function(){n<-0L;environment()}");
        let a_global = global(&a);
        let mut b = Frontend::default();
        bind(b.runtime_mut(), "f", function);
        let Sexp::Environment(frame) = last(&mut b, "g<-function()f();g()") else {
            panic!("callee did not export its execution frame")
        };
        assert_eq!(frame.parent().unwrap(), Some(a_global));
        let alias = frame.clone();
        drop(a);
        bind(b.runtime_mut(), "saved", Sexp::Environment(frame.clone()));
        bind(b.runtime_mut(), "alias", Sexp::Environment(alias.clone()));
        assert_eq!(
            last(&mut b, "saved$n<-41L;alias$n<-alias$n+1L;saved$n"),
            Sexp::Integer(vec![Some(42)])
        );
        drained(&mut b);
        drop(b);
        alias
            .insert("n".into(), &Sexp::Integer(vec![Some(iteration)]))
            .unwrap();
        assert_eq!(
            frame.get("n").unwrap(),
            Some(Sexp::Integer(vec![Some(iteration)]))
        );
        drop(alias);
        drop(frame);
    }
    // Final graph reclamation is proved separately by the graph-private StaleNode test.
}
