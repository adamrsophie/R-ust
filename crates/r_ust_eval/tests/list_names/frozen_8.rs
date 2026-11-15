use super::*;

#[test]
fn empty_classed_names() -> Result {
    run(
        r####"x<-list();class(x)<-"foo";names(x)<-character();list(value=x,names=names(x),attributes=attributes(x),class=class(x),names_absent=is.null(names(x)))"####,
        Sexp::list_with_metadata(
            vec![
                ListElement {
                    name: Some("value".into()),
                    value: Sexp::list_with_metadata(vec![], true, None),
                },
                ListElement {
                    name: Some("names".into()),
                    value: Sexp::String(vec![]),
                },
                ListElement {
                    name: Some("attributes".into()),
                    value: Sexp::list_with_metadata(
                        vec![
                            ListElement {
                                name: Some("class".into()),
                                value: Sexp::String(vec![Some("foo".into())]),
                            },
                            ListElement {
                                name: Some("names".into()),
                                value: Sexp::String(vec![]),
                            },
                        ],
                        true,
                        None,
                    ),
                },
                ListElement {
                    name: Some("class".into()),
                    value: Sexp::String(vec![Some("foo".into())]),
                },
                ListElement {
                    name: Some("names_absent".into()),
                    value: Sexp::Logical(vec![Some(false)]),
                },
            ],
            true,
            None,
        ),
        "",
    )
}

#[test]
fn class_remove_preserves_names() -> Result {
    run(
        r####"x<-list(1L);class(x)<-"foo";names(x)<-NA_character_;class(x)<-NULL;list(value=x,names=names(x),attributes=attributes(x),class=class(x),names_absent=is.null(names(x)))"####,
        Sexp::list_with_metadata(
            vec![
                ListElement {
                    name: Some("value".into()),
                    value: Sexp::list_with_metadata(
                        vec![ListElement {
                            name: None,
                            value: Sexp::Integer(vec![Some(1)]),
                        }],
                        true,
                        None,
                    ),
                },
                ListElement {
                    name: Some("names".into()),
                    value: Sexp::String(vec![None]),
                },
                ListElement {
                    name: Some("attributes".into()),
                    value: Sexp::list_with_metadata(
                        vec![ListElement {
                            name: Some("names".into()),
                            value: Sexp::String(vec![None]),
                        }],
                        true,
                        None,
                    ),
                },
                ListElement {
                    name: Some("class".into()),
                    value: Sexp::String(vec![Some("list".into())]),
                },
                ListElement {
                    name: Some("names_absent".into()),
                    value: Sexp::Logical(vec![Some(false)]),
                },
            ],
            true,
            None,
        ),
        "",
    )
}
