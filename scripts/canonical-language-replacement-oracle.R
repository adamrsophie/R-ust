options(keep.source = FALSE, warn = 1)
cases <- list(
  c("subset_lhs", "x[1L] <- y", "`<-`(`[`(x, 1L), y)"),
  c("matrix_lhs", "x[,1L] <- y", "`<-`(`[`(x, , 1L), y)"),
  c("extract_lhs", "x[[1L]] <- y", "`<-`(`[[`(x, 1L), y)"),
  c("dollar_symbol_lhs", "x$a <- y", "`<-`(`$`(x, a), y)"),
  c("dollar_string_lhs", "x$\"a\" <- y", "`<-`(`$`(x, \"a\"), y)"),
  c("names_lhs", "names(x) <- y", "`<-`(names(x), y)")
)
cat("id\tleft_source\tright_source\tidentical\n")
for (case in cases) {
  left <- parse(text = case[[2]], keep.source = FALSE)[[1]]
  right <- parse(text = case[[3]], keep.source = FALSE)[[1]]
  cat(case, identical(left, right), sep = "\t")
  cat("\n")
}
