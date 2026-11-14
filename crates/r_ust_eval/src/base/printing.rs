pub(super) const DEFINITIONS: &[(&str, &str)] = &[
    ("print", r#"function(x, ...) UseMethod("print")"#),
    (
        "print.default",
        r#"function(x, digits = NULL, quote = TRUE, na.print = NULL,
                          print.gap = NULL, right = FALSE, max = NULL,
			  width = NULL,
                          useSource = TRUE, ...)
{
    # Arguments are wrapped in another pairlist because we need to
    # forward them to recursive print() calls.
    args <- pairlist(
	digits = digits,
	quote = quote,
	na.print = na.print,
	print.gap = print.gap,
	right = right,
	max = max,
	width = width,
	useSource = useSource,
        ...
    )

    # Missing elements are not forwarded so we pass their
    # `missingness`. Also this helps decide whether to call show()
    # with S4 objects (if any argument print() is used instead).
    missings <- c(missing(digits), missing(quote), missing(na.print),
		  missing(print.gap), missing(right), missing(max),
		  missing(width),
		  missing(useSource))

    .Internal(print.default(x, args, missings))
}"#,
    ),
];
