use super::*;

#[test]
fn print_mixed_blank_na() -> Result {
    run(
        r####"x<-list(1L,2L);names(x)<-c("",NA_character_);{print(x);invisible(NULL)}"####,
        Sexp::Nil,
        "[[1]]\n[1] 1\n\n$<NA>\n[1] 2\n\n",
    )
}

#[test]
fn rep_len_named() -> Result {
    run(
        r####"x<-list(1L,2L);names(x)<-c(NA_character_,"");x<-rep_len(x,3L);list(value=x,names=names(x),attributes=attributes(x),class=class(x),names_absent=is.null(names(x)))"####,
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
                                name: None,
                                value: Sexp::Integer(vec![Some(2)]),
                            },
                            ListElement {
                                name: None,
                                value: Sexp::Integer(vec![Some(1)]),
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
