use super::*;

#[test]
fn subset_replace_extend() -> Result {
    run(
        r####"x<-list();names(x)<-character();x[2L]<-list(3L);list(value=x,names=names(x),attributes=attributes(x),class=class(x),names_absent=is.null(names(x)))"####,
        Sexp::list_with_metadata(
            vec![
                ListElement {
                    name: Some("value".into()),
                    value: Sexp::list_with_metadata(
                        vec![
                            ListElement {
                                name: Some("".into()),
                                value: Sexp::Nil,
                            },
                            ListElement {
                                name: Some("".into()),
                                value: Sexp::Integer(vec![Some(3)]),
                            },
                        ],
                        true,
                        None,
                    ),
                },
                ListElement {
                    name: Some("names".into()),
                    value: Sexp::String(vec![Some("".into()), Some("".into())]),
                },
                ListElement {
                    name: Some("attributes".into()),
                    value: Sexp::list_with_metadata(
                        vec![ListElement {
                            name: Some("names".into()),
                            value: Sexp::String(vec![Some("".into()), Some("".into())]),
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
fn subset_replace_delete() -> Result {
    run(
        r####"x<-list(1L);names(x)<-NA_character_;x[1L]<-NULL;list(value=x,names=names(x),attributes=attributes(x),class=class(x),names_absent=is.null(names(x)))"####,
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
                        vec![ListElement {
                            name: Some("names".into()),
                            value: Sexp::String(vec![]),
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
