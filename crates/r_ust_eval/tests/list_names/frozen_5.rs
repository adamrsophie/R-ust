use super::*;

#[test]
fn all_na_equal_identity() -> Result {
    run(
        r####"x<-list(1L);names(x)<-NA_character_;y<-x;identical(x,y)"####,
        Sexp::Logical(vec![Some(true)]),
        "",
    )
}

#[test]
fn copy_on_names_assignment() -> Result {
    run(
        r####"x<-list(1L);names(x)<-NA_character_;y<-x;names(y)<-"b";list(x=x,y=y,names_x=names(x),names_y=names(y))"####,
        Sexp::list_with_metadata(
            vec![
                ListElement {
                    name: Some("x".into()),
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
                    name: Some("y".into()),
                    value: Sexp::list_with_metadata(
                        vec![ListElement {
                            name: Some("b".into()),
                            value: Sexp::Integer(vec![Some(1)]),
                        }],
                        true,
                        None,
                    ),
                },
                ListElement {
                    name: Some("names_x".into()),
                    value: Sexp::String(vec![None]),
                },
                ListElement {
                    name: Some("names_y".into()),
                    value: Sexp::String(vec![Some("b".into())]),
                },
            ],
            true,
            None,
        ),
        "",
    )
}
