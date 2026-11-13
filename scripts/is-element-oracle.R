stopifnot(getRversion() == "4.6.0")
compiler::enableJIT(0L)
cases <- c(
  empty = "is.element(integer(0),1L)",
  null = "is.element(NULL,NULL)",
  empty_set = "is.element(c(1L,2L),NULL)",
  integers = "is.element(c(3L,1L,3L,2L),c(1L,3L))",
  doubles = "is.element(c(1.5,2,Inf,-Inf),c(2,Inf))",
  logicals = "is.element(c(TRUE,FALSE,NA),c(FALSE,NA))",
  mixed_numeric = "is.element(c(TRUE,FALSE),c(0L,2L))",
  missing = "is.element(c(NA_real_,NaN,1),c(NaN,1))",
  missing_table = "is.element(c(NA_real_,NaN,1),NA_real_)",
  strings = "is.element(c('a','b','a',NA_character_),c('a',NA_character_))",
  coercion = "is.element(c(1L,2L,NA_integer_),c('1','NA'))",
  factors = "is.element(factor(c('b','a','b')),factor(c('a','c')))",
  factor_character = "is.element(factor(c('b','a')),c('a','b'))",
  list = "is.element(list(1L,'a',NULL),list(1L,NULL))",
  list_scalars = "is.element(list(1L,2L),1L)",
  matrix = "is.element(matrix(1:4,2),c(2L,4L))",
  named = "is.element(set=c(2L,3L),el=c(1L,2L))",
  partial = "is.element(s=2L,e=c(1L,2L))",
  exact_before_position = "is.element(2L,el=c(1L,2L))",
  alias = "f<-is.element;f(c(1L,2L),2L)",
  dots = "f<-function(...)is.element(...);f(e=c(1L,2L),s=2L)",
  shadow_helper = "match<-function(...)stop('global match');is.element(c(1L,2L),2L)",
  shadow_callable = "is.element<-function(...)TRUE;is.element(stop('unforced'))",
  alias_before_shadow = "f<-is.element;is.element<-function(...)FALSE;f(1L,1L)",
  caller_bindings = "el<-1L;set<-2L;is.element(el,set)",
  force_order = "is.element({cat('EL');1L},{cat('SET');1L})",
  force_once = "n<-0L;is.element({n<<-n+1L;n},{cat(n);1L})",
  empty_forces_set = "is.element(NULL,{cat('SET');NULL})",
  missing_el = "is.element(set=1L)",
  missing_set = "is.element(1L)",
  missing_both = "is.element()",
  duplicate = "is.element(el=1L,el=2L,set=1L)",
  extra = "is.element(1L,1L,3L)",
  unused_named = "is.element(1L,1L,extra=3L)",
  unused_side_effect = "is.element(1L,1L,extra={cat('EXTRA');3L})",
  warning_partial = "options(warnPartialMatchArgs=TRUE,warn=1);is.element(e=1L,s=1L)",
  warning_error = "options(warnPartialMatchArgs=TRUE,warn=2);is.element(e=1L,set=1L)",
  caller_error = "is.element(stop('caller'),1L)",
  names = "is.element(c(a=1L,b=2L),2L)",
  object = "is.element(structure(c(1L,2L),class='custom'),2L)"
)
hex <- function(value) {
  if (!nzchar(value)) return('-')
  paste(format(charToRaw(enc2utf8(value))),collapse='')
}
lines <- sprintf('# %s %s LDOUBLE %s JIT 0 DIRECT_TOP_LEVEL',
                 R.version.string, R.version$platform, .Machine$sizeof.longdouble)
for (i in seq_along(cases)) {
  warnings <- character()
  error <- ''
  value <- NULL
  env <- globalenv()
  rm(list=intersect(c('match','is.element','f','el','set','n'),ls(env)),envir=env)
  old <- options()
  output <- utils::capture.output(value <- tryCatch(withCallingHandlers(
    eval(parse(text=cases[[i]]),envir=env),
    warning=function(w) {
      if (getOption('warn') == 2L) return()
      warnings <<- c(warnings,conditionMessage(w))
      invokeRestart('muffleWarning')
    }),error=function(e) {error <<- conditionMessage(e);NULL}))
  options(old)
  kind <- if(nzchar(error)) 'error' else typeof(value)
  data <- if(nzchar(error)) '-' else paste0('logical|',length(value),'|',paste(
    ifelse(is.na(value),'NA',ifelse(value,'T','F')),collapse=','))
  fields <- c(i,names(cases)[[i]],cases[[i]],kind,data,hex(error),
              hex(paste(warnings,collapse='\n')),hex(paste(output,collapse='\n')))
  lines <- c(lines,paste(fields,collapse='\t'))
}
dir.create('crates/r_ust_eval/tests/fixtures',recursive=TRUE,showWarnings=FALSE)
writeLines(lines,'crates/r_ust_eval/tests/fixtures/is-element-r-4.6.0.tsv')
