use super::*;

#[test]
fn dollar_blank() -> Result {
    run(
        r####"x<-list(1L,2L);names(x)<-c("",NA_character_);x<-x[[""]];list(value=x,names=names(x),attributes=attributes(x),class=class(x),names_absent=is.null(names(x)))"####,
        Sexp::list_with_metadata(
            vec![
                ListElement {
                    name: Some("value".into()),
                    value: Sexp::Nil,
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
                    value: Sexp::String(vec![Some("NULL".into())]),
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

#[test]
fn concat_default_nested() -> Result {
    run(
        r####"x<-list(1L,2L);names(x)<-c(NA_character_,"");x<-c(list(x),list());list(value=x,names=names(x),attributes=attributes(x),class=class(x),names_absent=is.null(names(x)))"####,
        Sexp::list_with_metadata(
            vec![
                ListElement {
                    name: Some("value".into()),
                    value: Sexp::list_with_metadata(
                        vec![ListElement {
                            name: None,
                            value: Sexp::list_with_metadata(
                                vec![
                                    ListElement {
                                        name: None,
                                        value: Sexp::Integer(vec![Some(1)]),
                                    },
                                    ListElement {
                                        name: Some("".into()),
                                        value: Sexp::Integer(vec![Some(2)]),
                                    },
                                ],
                                true,
                                None,
                            ),
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
