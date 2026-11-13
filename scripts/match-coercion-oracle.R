stopifnot(getRversion() == '4.6.0')
compiler::enableJIT(0L)
cases <- c(
  logical_integer = 'match(c(TRUE,FALSE,NA),c(0L,1L,NA_integer_))',
  integer_logical = 'match(c(1L,0L,2L),c(FALSE,TRUE))',
  logical_double = 'c(TRUE,FALSE,NA)%in%c(0,2,NA_real_)',
  double_logical = 'c(0,1,1.5)%in%c(TRUE,FALSE)',
  integer_character = 'match(c(1L,2L,NA_integer_),c("1","NA"),0L)',
  character_integer = 'match(c("1","2","NA",NA_character_),c(1L,NA_integer_),0L)',
  logical_character = 'match(c(TRUE,FALSE,NA),c("FALSE","TRUE",NA_character_))',
  character_logical = 'c("TRUE","FALSE","NA",NA_character_)%in%c(TRUE,NA)',
  double_character = 'match(c(1.5,2,Inf,-Inf,NaN,NA_real_),c("1.5","Inf","-Inf","NaN",NA_character_),0L)',
  character_double = 'c("1.5","NaN","NA",NA_character_)%in%c(1.5,NaN,NA_real_)',
  factor_integer = 'match(factor(c("2","1")),c(1L,2L))',
  integer_factor = 'c(1L,2L,3L)%in%factor(c("2","1"))',
  matrix_logical = 'match(matrix(c(TRUE,FALSE),1),c(0L,1L))',
  matrix_character = 'match(c(1L,2L),matrix(c("2","1"),1))',
  signed_zero = 'match(c(0,-0),c(-0,0))',
  signed_zero_in = 'c(0,-0)%in%c(-0)',
  signed_zero_unique = 'length(unique(c(0,-0)))',
  signed_zero_set = 'setequal(c(0),c(-0))',
  na_nan = 'match(c(NA_real_,NaN,NA_real_),c(NaN,NA_real_))',
  missing_literal = 'c(NA_character_,"NA","NaN")%in%c(NA_real_,NaN)'
)
lines <- sprintf('# %s %s LDOUBLE %s JIT 0 DIRECT_TOP_LEVEL',
                 R.version.string,R.version$platform,.Machine$sizeof.longdouble)
for (i in seq_along(cases)) {
  value <- eval(parse(text=cases[[i]]),envir=globalenv())
  data <- if (is.logical(value)) ifelse(value,'T','F') else as.character(value)
  data[is.na(value)] <- 'NA'
  fields <- c(i,names(cases)[[i]],cases[[i]],typeof(value),
    paste0(typeof(value),'|',length(value),'|',paste(data,collapse=',')),'-','-','-')
  lines <- c(lines,paste(fields,collapse='\t'))
}
dir.create('crates/r_ust_eval/tests/fixtures',recursive=TRUE,showWarnings=FALSE)
writeLines(lines,'crates/r_ust_eval/tests/fixtures/match-coercion-r-4.6.0.tsv')
