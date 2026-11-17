use super::*;

#[test]
fn atomic_and_aggregate_attributes_preserve_order_and_aliases() -> Result {
    for source in [
        "1:3",
        "c(1.5,2.5,3.5)",
        "c(TRUE,FALSE,NA)",
        "c('a','b','c')",
        "list(1,2,3)",
        "expression(a,b,c)",
    ] {
        let mut f = Frontend::default();
        truth(&mut f, &format!("x<-{source};y<-x;attr(y,'a')<-1L;attr(y,'b')<-2L;attr(y,'a')<-3L;is.null(attributes(x)) && identical(attributes(y),list(a=3L,b=2L))"))?;
        truth(
            &mut f,
            "attr(y,'a')<-NULL;attr(y,'a')<-4L;identical(names(attributes(y)),c('b','a'))",
        )?;
        truth(
            &mut f,
            "attributes(y)<-NULL;is.null(attributes(y)) && identical(x,y)",
        )?;
    }
    Ok(())
}

#[test]
fn names_padding_na_empty_and_removal() -> Result {
    let mut f = Frontend::default();
    truth(&mut f, "x<-1:3;names(x)<-'a';identical(names(x),c('a',NA_character_,NA_character_)) && typeof(x)=='integer'")?;
    truth(
        &mut f,
        "names(x)<-character();identical(names(x),rep(NA_character_,3))",
    )?;
    truth(
        &mut f,
        "names(x)<-c('',NA,'z');identical(names(x),c('',NA_character_,'z'))",
    )?;
    truth(&mut f, "names(x)<-NULL;is.null(attributes(x))")?;
    truth(
        &mut f,
        "attr(x,'names')<-1:2;identical(names(x),c('1','2',NA_character_))",
    )?;
    Ok(())
}

#[test]
fn dimensions_first_and_one_dimensional_names_distinction() -> Result {
    let mut f = Frontend::default();
    truth(&mut f, "x<-1:4;attributes(x)<-list(note=7L,dimnames=list(c('a','b'),c('c','d')),dim=c(2L,2L));identical(names(attributes(x)),c('dim','note','dimnames')) && identical(class(x),c('matrix','array')) && !is.object(x)")?;
    truth(&mut f, "x<-structure(1:3,dim=3L,dimnames=list(c('a','b','c')));attr(x,'names')<-NULL;identical(names(x),c('a','b','c'))")?;
    truth(
        &mut f,
        "names(x)<-NULL;is.null(attr(x,'dimnames')) && identical(attr(x,'dim'),3L)",
    )?;
    truth(
        &mut f,
        "names(x)<-'a';identical(attr(x,'dimnames'),list(c('a',NA_character_,NA_character_)))",
    )?;
    truth(&mut f, "attr(x,'dim')<-NULL;is.null(attributes(x))")?;
    Ok(())
}

#[test]
fn structure_aliases_factor_rows_and_object_metadata() -> Result {
    let mut f = Frontend::default();
    truth(&mut f, "x<-structure(1:3,.Names=c('a','b','c'),.Label=c('a','b','c'),class='factor');typeof(x)=='integer' && is.object(x) && identical(attr(x,'levels'),c('a','b','c')) && identical(names(x),c('a','b','c'))")?;
    truth(&mut f, "y<-unclass(x);!is.object(y) && is.object(x) && identical(attr(y,'levels'),attr(x,'levels'))")?;
    truth(&mut f, "x<-list(a=1:3);attr(x,'row.names')<-c(NA_integer_,-3L);identical(attr(x,'row.names'),1:3) && identical(attributes(x)$row.names,1:3)")?;
    truth(&mut f, "x<-structure(1:3,class='custom',note=7L);attr(x,'class')<-character();!is.object(x) && identical(class(x),'integer') && identical(attributes(x),list(note=7L))")?;
    Ok(())
}

#[test]
fn duplicate_attributes_and_null_replacement() -> Result {
    let mut f = Frontend::default();
    truth(&mut f, "x<-1:3;attributes(x)<-list(note=1L,note=2L,class='a',class='b');identical(attributes(x),list(note=2L,class='b'))")?;
    truth(&mut f, "x<-NULL;attr(x,'a')<-NULL;is.null(x)")?;
    truth(&mut f, "attributes(x)<-list();identical(x,list())")?;
    truth(&mut f, "x<-1:3;a<-list(1L,2L);names(a)<-c(NA,'a');attributes(x)<-a;identical(names(attributes(x)),c('NA','a'))")?;
    Ok(())
}

#[test]
fn storage_conversion_preserves_all_attributes_and_aliases() -> Result {
    let mut f = Frontend::default();
    truth(&mut f, "x<-structure(c(1.5,2.5),names=c('a','b'),note=7L);y<-x;storage.mode(y)<-'integer';typeof(y)=='integer' && typeof(x)=='double' && identical(attributes(x),attributes(y)) && identical(as.integer(y),c(1L,2L))")?;
    truth(&mut f, "storage.mode(y)<-'character';typeof(y)=='character' && identical(attributes(x),attributes(y))")?;
    truth(
        &mut f,
        "storage.mode(y)<-'double';typeof(y)=='double' && identical(attributes(x),attributes(y))",
    )?;
    Ok(())
}
