"""Frozen small programs from the approved D1 prerequisite QA boundary."""
from dataclasses import dataclass


@dataclass(frozen=True, slots=True)
class Case:
    identifier: str
    program: tuple[str, str]
    frame: str = "none"

    @property
    def source(self) -> str:
        setup, expression = self.program
        return (setup + ";" if setup else "") + expression


CLASS_SETUP = "x<-structure(list(message='FAIL',call=NULL),class=c('simpleError','error','condition'))"
CASES = (
    Case("class_names_type", (CLASS_SETUP, "c(typeof(x),class(x),names(x))")),
    Case("attribute_creation_order", ("x<-structure(list(1L),class='box',names='a')", "names(attributes(x))")),
    Case("attribute_remove_order", ("x<-structure(list(1L),class='box',names='a');class(x)<-NULL", "names(attributes(x))")),
    Case("attribute_readd_order", ("x<-structure(list(1L),class='box',names='a');class(x)<-NULL;class(x)<-'box'", "names(attributes(x))")),
    Case("class_message", (CLASS_SETUP, "x$message")),
    Case("class_extract", (CLASS_SETUP, "x[[1L]]")),
    Case("pairlist_tagged", ("", "pairlist(a=1L,b=NULL)")),
    Case("pairlist_empty", ("", "pairlist()")),
    Case("pairlist_conversion", ("", "as.pairlist(list(a=1L,b=NULL))")),
    Case("null_attribute", ("", "structure(NULL,class='x')"), "stop"),
    Case("print_default_depth", ("f<-function(x=c(sys.nframe(),sys.parent())){g<-function(y)y;g(x)}", "print(f())")),
    Case("generic_transfer", ("g<-function(x,...) {UseMethod('g');stop('UNREACHABLE')};g.integer<-function(x,...)c(sys.nframe(),sys.parent(),missing(x));g.default<-function(x,...)99L", "g(1L)")),
    Case("print_alias", ("p<-print", "p(1L)")),
    Case("print_computed", ("", "(function()print)()(1L)")),
    Case("print_quote", ("", "print.default('x',quote=FALSE)")),
    Case("dispatch_missing_generic", ("", "UseMethod()"), "arity"),
    Case("dispatch_invalid_generic", ("", "UseMethod(1)"), "type"),
    Case("dispatch_context", ("", "UseMethod('g')"), "domain"),
    Case("print_invalid_quote", ("", "print.default(1,quote=NA)"), "domain"),
    Case("stack_restored_error", ("f<-function()stop('FAIL')", " c(inherits(tryCatch(f(),error=function(e)e),'error'),sys.nframe())")),
    Case("trycatch_plain", ("", "tryCatch(7L)")),
    Case("trycatch_finally", ("", "tryCatch(7L,finally=cat('F'))")),
    Case("trycatch_error_finally", ("", "tryCatch(stop('A'),error=function(e)e$message,finally=cat('F'))")),
    Case("trycatch_precedence", ("", "tryCatch(stop('A'),simpleError=function(e)'S',error=function(e)'E')")),
    Case("trycatch_condition", ("", "tryCatch(stop(simpleError('C')),error=function(e)e$message)")),
    Case("calling_then_exiting", ("", "tryCatch(withCallingHandlers(stop('A'),error=function(e)cat('CALL')),error=function(e){cat('EXIT');e$message})")),
    Case("cleanup_message", ("f<-function(){on.exit(cat('CLEAN'));stop('ORIGINAL')}", "tryCatch(f(),error=function(e)e$message)")),
    Case("unnamed_handler", ("", "tryCatch(1,function(e)e)"), "stop"),
    Case("bad_handler_data", ("", ".Internal(.addCondHands(1,list(function(e)e),globalenv(),NULL,FALSE))"), "domain"),
    Case("handler_invisible", ("", "tryCatch(stop('A'),error=function(e)invisible(9L))")),
    Case("handler_error", ("", "tryCatch(stop('A'),error=function(e)stop('B'))"), "stop"),
    Case("handler_no_match", ("", "tryCatch(stop('A'),warning=function(e)7L)"), "stop"),
    Case("cleanup_nested_error", ("f<-function(){on.exit(tryCatch(stop('SECOND'),error=function(e)cat(e$message)));stop('ORIGINAL')}", "tryCatch(f(),error=function(e)e$message)")),
)
