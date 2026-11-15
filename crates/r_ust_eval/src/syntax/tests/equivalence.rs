use super::*;

#[test]
fn pinned_reference_values_and_visibility() {
    let list = |name: Option<&str>, value| {
        Sexp::List(vec![ListElement {
            name: name.map(str::to_owned),
            value: integer(value),
        }])
    };
    let cases = vec![
        ("call_arithmetic", "", "1L+2L*3L", integer(7), true),
        ("call_unary", "", "-2L", integer(-2), true),
        ("call_parentheses", "", "(3L)", integer(3), true),
        ("call_braces", "", "{x<-1L;x+2L}", integer(3), true),
        (
            "call_if_false",
            "",
            "if(FALSE) stop(\"BAD\") else 7L",
            integer(7),
            true,
        ),
        ("call_if_no_else", "", "if(FALSE) 7L", Sexp::Nil, false),
        (
            "call_short_and",
            "",
            "FALSE && stop(\"BAD\")",
            logical(false),
            true,
        ),
        (
            "call_short_or",
            "",
            "TRUE || stop(\"BAD\")",
            logical(true),
            true,
        ),
        ("call_assignment", "", "x<-4L;x", integer(4), true),
        ("call_equal_assignment", "", "{x=5L};x", integer(5), true),
        ("call_right_assignment", "", "6L -> x;x", integer(6), true),
        (
            "call_subset_hole",
            "x<-1:3",
            "x[]",
            Sexp::Integer(vec![Some(1), Some(2), Some(3)]),
            true,
        ),
        (
            "call_extract",
            "x<-list(3L,7L)",
            "x[[2L]]",
            integer(7),
            true,
        ),
        ("call_dollar", "x<-list(a=8L)", "x$a", integer(8), true),
        (
            "call_subset_replace",
            "x<-1:3",
            "x[2L]<-9L;x",
            Sexp::Integer(vec![Some(1), Some(9), Some(3)]),
            true,
        ),
        (
            "call_extract_replace",
            "x<-list(1L)",
            "x[[1L]]<-9L;x",
            list(None, 9),
            true,
        ),
        (
            "call_dollar_replace",
            "x<-list(a=1L)",
            "x$a<-9L;x",
            list(Some("a"), 9),
            true,
        ),
        (
            "call_names_replace",
            "x<-list(1L)",
            "names(x)<-\"z\";x",
            list(Some("z"), 1),
            true,
        ),
        (
            "call_computed_head",
            "",
            "(function(x) x+1L)(8L)",
            integer(9),
            true,
        ),
        (
            "call_nonfunction_skip",
            "sum<-42L",
            "sum(1L,2L)",
            integer(3),
            true,
        ),
        (
            "call_default_lazy",
            "",
            "(function(x=stop(\"BAD\")) 1L)()",
            integer(1),
            true,
        ),
        (
            "call_named_arguments",
            "",
            "(function(x,y) c(x,y))(y=2L,x=1L)",
            Sexp::Integer(vec![Some(1), Some(2)]),
            true,
        ),
        (
            "call_missing_default",
            "",
            "(function(x=9L) x)(x=)",
            integer(9),
            true,
        ),
        (
            "call_function_source_null",
            "",
            "f<-function(x=2L)x+1L;c(f(),identical(environment(f),globalenv()))",
            Sexp::Integer(vec![Some(3), Some(1)]),
            true,
        ),
        (
            "call_quote_nonforce",
            "",
            "quote(stop(\"BAD\"))",
            canonical_language(&expression("stop(\"BAD\")")).unwrap(),
            true,
        ),
        ("call_invisible", "", "invisible(7L)", integer(7), false),
        (
            "call_brace_head_changes",
            "",
            "{`+`<-function(x,y) 81L;1L+2L}",
            integer(81),
            true,
        ),
        (
            "call_for_break_next",
            "",
            "{x<-0L;for(i in 1:5){if(i==2L) next;if(i==4L) break;x<-x+i};x}",
            integer(4),
            true,
        ),
        (
            "call_while",
            "",
            "{i<-0L;while(i<3L)i<-i+1L;i}",
            integer(3),
            true,
        ),
        (
            "call_repeat",
            "",
            "{i<-0L;repeat {i<-i+1L;if(i==3L)break};i}",
            integer(3),
            true,
        ),
    ];
    for (id, setup, subject, expected, visible) in cases {
        for canonical in [false, true] {
            let mut runtime = RuntimeState::new(Default::default());
            source(&mut runtime, setup, canonical);
            let result = source(&mut runtime, subject, canonical);
            check_value(&result, &expected, id);
            assert_eq!(runtime.visible, visible, "{id}, canonical={canonical}");
        }
    }
}

#[test]
fn pinned_rebound_syntax_receives_lazy_arguments() {
    let cases = [
        (
            "call_rebound_plus",
            "+",
            "function(x,y) x",
            "1L+stop(\"BAD\")",
            1,
        ),
        (
            "call_rebound_if",
            "if",
            "function(...) 73L",
            "if(stop(\"BAD\")) 1L else 2L",
            73,
        ),
        (
            "call_rebound_function",
            "function",
            "function(...) 74L",
            "function(x) stop(\"BAD\")",
            74,
        ),
        (
            "call_rebound_brace",
            "{",
            "function(...) 75L",
            "{stop(\"BAD\");1L}",
            75,
        ),
        (
            "call_rebound_paren",
            "(",
            "function(x) 76L",
            "(stop(\"BAD\"))",
            76,
        ),
        (
            "call_rebound_assign",
            "<-",
            "function(...) 77L",
            "x<-stop(\"BAD\")",
            77,
        ),
        (
            "call_rebound_for",
            "for",
            "function(...) 78L",
            "for(i in stop(\"BAD\")) stop(\"BAD\")",
            78,
        ),
    ];
    for (id, name, replacement, subject, expected) in cases {
        for canonical in [false, true] {
            let mut runtime = RuntimeState::new(Default::default());
            runtime.current = runtime.current.child().unwrap();
            let function = source(&mut runtime, replacement, false);
            runtime.current.insert(name.into(), &function).unwrap();
            let result = source(&mut runtime, subject, canonical);
            check_value(&result, &integer(expected), id);
            assert!(runtime.visible, "{id} preserves closure visibility");
        }
    }
}
