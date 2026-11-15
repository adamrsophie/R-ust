use super::Row;
pub(super) fn rows() -> Vec<Row> {
    vec![
        Row {
            id: "restart_quit_cleanup",
            setup: "",
            source: "f<-function(){on.exit(cat('QUIT_CLEANUP|'));quit(save='no',status=4L,runLast=FALSE)};withRestarts(f(),r=function()0L)",
            binding: "",
            replacement: "",
            error_message: "",
            status: 4,
            stdout: include_str!("../raw/restart_quit_cleanup.stdout"),
            stderr: include_str!("../raw/restart_quit_cleanup.stderr"),
        },
        Row {
            id: "do_call_lexical_lookup",
            setup: "",
            source: "f<-function(x)x+1L;e<-new.env();e$x<-20L;do.call('f',list(quote(x)),envir=e)",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/do_call_lexical_lookup.stdout"),
            stderr: include_str!("../raw/do_call_lexical_lookup.stderr"),
        },
        Row {
            id: "do_call_quote_true",
            setup: "",
            source: "x<-9L;do.call(identity,list(quote(x)),quote=TRUE)",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/do_call_quote_true.stdout"),
            stderr: include_str!("../raw/do_call_quote_true.stderr"),
        },
        Row {
            id: "do_call_lazy_argument",
            setup: "",
            source: "do.call(function(x,y)x,list(7L,quote(stop('NOT_FORCED'))))",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/do_call_lazy_argument.stdout"),
            stderr: include_str!("../raw/do_call_lazy_argument.stderr"),
        }
    ]
}
