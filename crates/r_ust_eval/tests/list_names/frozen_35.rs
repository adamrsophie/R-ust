use super::*;

#[test]
fn subset_delete_classfirst() -> Result {
    run(
        r####"x<-list(1L);class(x)<-"foo";names(x)<-NA_character_;x[1L]<-NULL;list(value=x,names=names(x),attributes=attributes(x),class=class(x),names_absent=is.null(names(x)))"####,
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
                                name: Some("names".into()),
                                value: Sexp::String(vec![]),
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

#[test]
fn named_extend_classfirst() -> Result {
    run(
        r####"x<-list(1L);class(x)<-"foo";names(x)<-NA_character_;x$z<-2L;list(value=x,names=names(x),attributes=attributes(x),class=class(x),names_absent=is.null(names(x)))"####,
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
                                name: Some("z".into()),
                                value: Sexp::Integer(vec![Some(2)]),
                            },
                        ],
                        true,
                        None,
                    ),
                },
                ListElement {
                    name: Some("names".into()),
                    value: Sexp::String(vec![None, Some("z".into())]),
                },
                ListElement {
                    name: Some("attributes".into()),
                    value: Sexp::list_with_metadata(
                        vec![
                            ListElement {
                                name: Some("names".into()),
                                value: Sexp::String(vec![None, Some("z".into())]),
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
