use super::*;

#[test]
fn concat_outer_name_mixed() -> Result {
    run(
        r####"x<-list(1L,2L,3L);names(x)<-c(NA_character_,"","x");x<-c(prefix=x);list(value=x,names=names(x),attributes=attributes(x),class=class(x),names_absent=is.null(names(x)))"####,
        Sexp::list_with_metadata(
            vec![
                ListElement {
                    name: Some("value".into()),
                    value: Sexp::list_with_metadata(
                        vec![
                            ListElement {
                                name: Some("prefix.NA".into()),
                                value: Sexp::Integer(vec![Some(1)]),
                            },
                            ListElement {
                                name: Some("prefix2".into()),
                                value: Sexp::Integer(vec![Some(2)]),
                            },
                            ListElement {
                                name: Some("prefix.x".into()),
                                value: Sexp::Integer(vec![Some(3)]),
                            },
                        ],
                        true,
                        None,
                    ),
                },
                ListElement {
                    name: Some("names".into()),
                    value: Sexp::String(vec![
                        Some("prefix.NA".into()),
                        Some("prefix2".into()),
                        Some("prefix.x".into()),
                    ]),
                },
                ListElement {
                    name: Some("attributes".into()),
                    value: Sexp::list_with_metadata(
                        vec![ListElement {
                            name: Some("names".into()),
                            value: Sexp::String(vec![
                                Some("prefix.NA".into()),
                                Some("prefix2".into()),
                                Some("prefix.x".into()),
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
fn concat_drop_names() -> Result {
    run(
        r####"x<-list(1L);names(x)<-NA_character_;x<-c(x,use.names=FALSE);list(value=x,names=names(x),attributes=attributes(x),class=class(x),names_absent=is.null(names(x)))"####,
        Sexp::list_with_metadata(
            vec![
                ListElement {
                    name: Some("value".into()),
                    value: Sexp::list_with_metadata(
                        vec![ListElement {
                            name: None,
                            value: Sexp::Integer(vec![Some(1)]),
                        }],
                        false,
                        None,
                    ),
                },
                ListElement {
                    name: Some("names".into()),
                    value: Sexp::Nil,
                },
                ListElement {
                    name: Some("attributes".into()),
                    value: Sexp::Nil,
                },
                ListElement {
                    name: Some("class".into()),
                    value: Sexp::String(vec![Some("list".into())]),
                },
                ListElement {
                    name: Some("names_absent".into()),
                    value: Sexp::Logical(vec![Some(true)]),
                },
            ],
            true,
            None,
        ),
        "",
    )
}
