use super::*;

pub(super) fn run() -> Result {
    let mut f = Frontend::default();
    f.eval_source("x<-structure(c('12','2',NA_character_),previous=c('first','second'),custom=list(1L,'z')); plain<-c('12','2',NA_character_)")?;
    for operation in [
        "x[1L]",
        "x[[1L]]",
        "as.character(x)",
        "as.integer(x)",
        "as.double(x)",
        "as.logical(x)",
        "c(x)",
        "rep(x,2L)",
        "rep_len(x,5L)",
        "as.list(x)",
        "is.na(x)",
        "rev(x)",
        "sort(x)",
        "strrep(x,2L)",
        "nchar(x)",
        "nzchar(x)",
        "strtoi(x)",
        "match(x,plain)",
        "pmatch(x,plain)",
        "charmatch(x,plain)",
        "x=='12'",
        "unique(x)",
        "duplicated(x)",
        "append(x,'3')",
        "ifelse(c(TRUE,FALSE,TRUE),x,x)",
    ] {
        truth(&mut f, &format!("is.null(attributes({operation}))"))?;
        let plain = operation.replace("x", "plain");
        truth(&mut f, &format!("identical({operation},{plain})"))?;
    }
    for operation in [
        "x[]",
        "unlist(x)",
        "toupper(x)",
        "tolower(x)",
        "substr(x,1L,1L)",
        "substring(x,1L,1L)",
        "chartr('1','3',x)",
        "strtrim(x,1L)",
        "trimws(x)",
        "sub('1','3',x)",
        "gsub('1','3',x)",
        "encodeString(x)",
        "replace(x,1L,'z')",
    ] {
        truth(
            &mut f,
            &format!("identical(attributes({operation}),attributes(x))"),
        )?;
    }
    for operation in [
        "y[1L]<-'z'",
        "y[[1L]]<-'z'",
        "y[5L]<-'z'",
        "y[[5L]]<-'z'",
        "y[]<-'z'",
    ] {
        truth(
            &mut f,
            &format!("y<-x;{operation};identical(attributes(y),attributes(x))"),
        )?;
    }
    truth(
        &mut f,
        "y<-x;y[5L]<-'z';length(y)==5L && is.na(y[4L]) && identical(y[5L],'z')",
    )?;
    truth(&mut f,"y<-structure(character(),custom=1L); y[2L]<-'z'; identical(attributes(y),list(custom=1L)) && is.na(y[1L])")?;
    for source in [
        "anyNA(x) && identical(lengths(x),c(1L,1L,1L))",
        "identical(strtoi(structure('12',custom=1L)),12L)",
        "identical(utf8ToInt(structure('A',custom=1L)),65L)",
        "identical(pmatch(structure('a',custom=1L),c('abc','def')),1L)",
        "identical(charmatch(structure('a',custom=1L),c('abc','def')),1L)",
        "identical(list(a=1L)[[structure('a',custom=1L)]],1L)",
        "identical(list(a=1L)[structure('a',custom=1L)],list(a=1L))",
        "identical(inherits(x,structure('character',custom=1L)),TRUE)",
        "identical(attr(x,structure('custom',tag=1L)),list(1L,'z'))",
        "identical(attr(x,'cust'),list(1L,'z')) && is.null(attr(x,'cust',exact=TRUE)) && is.null(attr(x,'cust',TRUE))",
    ] {
        truth(&mut f, source)?;
    }
    f.runtime_mut().take_output();
    f.eval_source(include_str!("display.R"))?;
    assert_eq!(stdout(&mut f), include_str!("display.expected"));
    Ok(())
}
