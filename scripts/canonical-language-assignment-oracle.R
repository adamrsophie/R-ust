# Parent supplemental assignment provenance oracle, R 4.6.0.
# Rscript --vanilla scripts/canonical-language-assignment-oracle.R
options(keep.source=FALSE, warn=1)
cat("id\tleft_source\tright_source\tidentical\n")
cat("variable\t{x=1L}\t{x<-1L}\t", identical(quote({x=1L}),quote({x<-1L})), "\n", sep="")
cat("subset\t{x[1L]=y}\t{x[1L]<-y}\t", identical(quote({x[1L]=y}),quote({x[1L]<-y})), "\n", sep="")
cat("matrix_subset\t{x[,1L]=y}\t{x[,1L]<-y}\t", identical(quote({x[,1L]=y}),quote({x[,1L]<-y})), "\n", sep="")
cat("extract\t{x[[1L]]=y}\t{x[[1L]]<-y}\t", identical(quote({x[[1L]]=y}),quote({x[[1L]]<-y})), "\n", sep="")
cat("dollar_symbol\t{x$a=y}\t{x$a<-y}\t", identical(quote({x$a=y}),quote({x$a<-y})), "\n", sep="")
cat("dollar_string\t{x$\"a\"=y}\t{x$\"a\"<-y}\t", identical(quote({x$"a"=y}),quote({x$"a"<-y})), "\n", sep="")
cat("names\t{names(x)=y}\t{names(x)<-y}\t", identical(quote({names(x)=y}),quote({names(x)<-y})), "\n", sep="")
