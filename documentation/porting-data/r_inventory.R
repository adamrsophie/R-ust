pkgs <- c("base","stats","utils","methods","graphics","grDevices","datasets","tools","parallel","compiler","grid","splines","stats4","tcltk")
rows <- list()
for (p in pkgs) {
  ok <- suppressWarnings(suppressPackageStartupMessages(requireNamespace(p, quietly = TRUE)))
  if (!ok) next
  if (p == "base") { env <- baseenv(); nms <- ls(env, all.names = TRUE) }
  else if (p == "datasets") { env <- as.environment("package:datasets"); nms <- ls(env, all.names = TRUE) }
  else { env <- asNamespace(p); nms <- getNamespaceExports(p) }
  nms <- sort(unique(nms))
  for (n in nms) {
    obj <- tryCatch(get(n, envir = env), error = function(e) NULL)
    kind <- if (is.null(obj)) "unknown" else if (is.primitive(obj)) (if (typeof(obj) == "special") "special" else "builtin") else if (is.function(obj)) "closure" else paste0("data:", class(obj)[1])
    fm <- ""
    if (is.function(obj)) {
      a <- tryCatch(formals(args(obj)), error = function(e) NULL)
      if (is.null(a) && !is.primitive(obj)) a <- formals(obj)
      if (!is.null(a)) fm <- paste(names(a), collapse = ",")
    }
    if (is.function(obj) && !is.primitive(obj) && any(grepl("UseMethod(", deparse(body(obj)), fixed = TRUE))) {
      ns <- if (p %in% c("base", "datasets")) baseenv() else asNamespace(p)
      dm <- tryCatch(get(paste0(n, ".default"), envir = ns), error = function(e) NULL)
      if (is.null(dm)) dm <- tryCatch(getS3method(n, "default"), error = function(e) NULL)
      if (is.function(dm)) fm <- paste(unique(c(strsplit(fm, ",")[[1]], names(formals(dm)))), collapse = ",")
    }
    internal <- ""
    if (is.function(obj) && !is.primitive(obj)) {
      b <- paste(deparse(body(obj)), collapse = " ")
      if (grepl(".Internal(", b, fixed = TRUE)) internal <- "internal"
    }
    rows[[length(rows) + 1]] <- c(p, n, kind, fm, internal)
  }
}
m <- do.call(rbind, rows)
write.table(m, file = commandArgs(TRUE)[1], sep = "\t", quote = FALSE, row.names = FALSE, col.names = c("package","name","kind","formals","internal"))
cat(R.version.string, "\n")
print(table(m[,1]))
