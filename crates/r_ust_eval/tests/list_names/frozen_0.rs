use super::*;

#[test]
fn empty_absent() -> Result {
    run(
        r####"x<-list();list(value=x,names=names(x),attributes=attributes(x),class=class(x),names_absent=is.null(names(x)))"####,
        Sexp::list_with_metadata(
            vec![
                ListElement {
                    name: Some("value".into()),
                    value: Sexp::list_with_metadata(vec![], false, None),
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

#[test]
fn empty_present() -> Result {
    run(
        r####"x<-list();names(x)<-character();list(value=x,names=names(x),attributes=attributes(x),class=class(x),names_absent=is.null(names(x)))"####,
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
