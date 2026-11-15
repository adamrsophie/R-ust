use super::*;

#[test]
fn subset_delete_oob() -> Result {
    run(
        r####"x<-list(1L,2L);names(x)<-c(NA_character_,"");x[c(1L,5L)]<-NULL;list(value=x,names=names(x),attributes=attributes(x),class=class(x),names_absent=is.null(names(x)))"####,
        Sexp::list_with_metadata(
            vec![
                ListElement {
                    name: Some("value".into()),
                    value: Sexp::list_with_metadata(
                        vec![
                            ListElement {
                                name: Some("".into()),
                                value: Sexp::Integer(vec![Some(2)]),
                            },
                            ListElement {
                                name: Some("".into()),
                                value: Sexp::Nil,
                            },
                            ListElement {
                                name: Some("".into()),
                                value: Sexp::Nil,
                            },
                        ],
                        true,
                        None,
                    ),
                },
                ListElement {
                    name: Some("names".into()),
                    value: Sexp::String(vec![Some("".into()), Some("".into()), Some("".into())]),
                },
                ListElement {
                    name: Some("attributes".into()),
                    value: Sexp::list_with_metadata(
                        vec![ListElement {
                            name: Some("names".into()),
                            value: Sexp::String(vec![
                                Some("".into()),
                                Some("".into()),
                                Some("".into()),
                            ]),
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

#[test]
fn numeric_extend_classfirst() -> Result {
    run(
        r####"x<-list(1L);class(x)<-"foo";names(x)<-NA_character_;x[[3L]]<-2L;list(value=x,names=names(x),attributes=attributes(x),class=class(x),names_absent=is.null(names(x)))"####,
        Sexp::list_with_metadata(
            vec![
                ListElement {
                    name: Some("value".into()),
                    value: Sexp::list_with_metadata(
                        vec![
                            ListElement {
                                name: None,
                                value: Sexp::Integer(vec![Some(1)]),
                            },
                            ListElement {
                                name: Some("".into()),
                                value: Sexp::Nil,
                            },
                            ListElement {
                                name: Some("".into()),
                                value: Sexp::Integer(vec![Some(2)]),
                            },
                        ],
                        true,
                        None,
                    ),
                },
                ListElement {
                    name: Some("names".into()),
                    value: Sexp::String(vec![None, Some("".into()), Some("".into())]),
                },
                ListElement {
                    name: Some("attributes".into()),
                    value: Sexp::list_with_metadata(
                        vec![
                            ListElement {
                                name: Some("names".into()),
                                value: Sexp::String(vec![None, Some("".into()), Some("".into())]),
                            },
                            ListElement {
                                name: Some("class".into()),
                                value: Sexp::String(vec![Some("foo".into())]),
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
