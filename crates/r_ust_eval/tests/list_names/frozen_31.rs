use super::*;

#[test]
fn lapply_classed() -> Result {
    run(
        r####"x<-list(1L,2L);names(x)<-c(NA_character_,"");class(x)<-"foo";x<-lapply(x,identity);list(value=x,names=names(x),attributes=attributes(x),class=class(x),names_absent=is.null(names(x)))"####,
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
                                value: Sexp::Integer(vec![Some(2)]),
                            },
                        ],
                        true,
                        None,
                    ),
                },
                ListElement {
                    name: Some("names".into()),
                    value: Sexp::String(vec![None, Some("".into())]),
                },
                ListElement {
                    name: Some("attributes".into()),
                    value: Sexp::list_with_metadata(
                        vec![ListElement {
                            name: Some("names".into()),
                            value: Sexp::String(vec![None, Some("".into())]),
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
fn unique_all_na() -> Result {
    run(
        r####"x<-list(1L,1L);names(x)<-c(NA_character_,NA_character_);x<-unique(x);list(value=x,names=names(x),attributes=attributes(x),class=class(x),names_absent=is.null(names(x)))"####,
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
