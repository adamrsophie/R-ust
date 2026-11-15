use super::*;

#[test]
fn print_empty_named() -> Result {
    run(
        r####"x<-list();names(x)<-character();{print(x);invisible(NULL)}"####,
        Sexp::Nil,
        "named list()\n",
    )
}

#[test]
fn print_all_na() -> Result {
    run(
        r####"x<-list(1L);names(x)<-NA_character_;{print(x);invisible(NULL)}"####,
        Sexp::Nil,
        "$<NA>\n[1] 1\n\n",
    )
}
