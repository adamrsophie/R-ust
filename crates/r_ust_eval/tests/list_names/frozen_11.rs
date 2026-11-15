use super::*;

#[test]
fn subset_character() -> Result {
    run(
        r####"x<-list(1L,2L,3L);names(x)<-c(NA_character_,"","NA");x<-x[c("NA","",NA_character_,"no")];list(value=x,names=names(x),attributes=attributes(x),class=class(x),names_absent=is.null(names(x)))"####,
        Sexp::list_with_metadata(
            vec![
                ListElement {
                    name: Some("value".into()),
                    value: Sexp::list_with_metadata(
                        vec![
                            ListElement {
                                name: Some("NA".into()),
                                value: Sexp::Integer(vec![Some(3)]),
                            },
                            ListElement {
                                name: None,
                                value: Sexp::Nil,
                            },
                            ListElement {
                                name: None,
                                value: Sexp::Nil,
                            },
                            ListElement {
                                name: None,
                                value: Sexp::Nil,
                            },
                        ],
                        true,
                        None,
                    ),
                },
                ListElement {
                    name: Some("names".into()),
                    value: Sexp::String(vec![Some("NA".into()), None, None, None]),
                },
                ListElement {
                    name: Some("attributes".into()),
                    value: Sexp::list_with_metadata(
                        vec![ListElement {
                            name: Some("names".into()),
                            value: Sexp::String(vec![Some("NA".into()), None, None, None]),
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
fn subset_unnamed_character() -> Result {
    run(
        r####"x<-list(1L);x<-x[c("a","")];list(value=x,names=names(x),attributes=attributes(x),class=class(x),names_absent=is.null(names(x)))"####,
        Sexp::list_with_metadata(
            vec![
                ListElement {
                    name: Some("value".into()),
                    value: Sexp::list_with_metadata(
                        vec![
                            ListElement {
                                name: None,
                                value: Sexp::Nil,
                            },
                            ListElement {
                                name: None,
                                value: Sexp::Nil,
                            },
                        ],
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
