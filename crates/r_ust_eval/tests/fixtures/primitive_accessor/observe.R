args <- commandArgs(TRUE)
stopifnot(length(args) == 3L)
source(file.path(args[[1L]], "reference-helpers.R"))
profile()
rows <- read_cases(args[[2L]])
row <- rows[rows$id == args[[3L]], ]
stopifnot(nrow(row) == 1L, row$expected_status == "0")
environment <- globalenv()
invisible(eval(parse(text = unhex(row$setup_hex), keep.source = FALSE), environment))
warnings <- character()
effects <- utils::capture.output({
    observed <- withCallingHandlers(
        withVisible(eval(parse(text = unhex(row$subject_hex), keep.source = FALSE), environment)),
        warning = function(condition) {
            warnings <<- c(warnings, paste(class(condition), collapse = ","),
                          conditionMessage(condition), paste(deparse(conditionCall(condition)), collapse = "\n"))
            invokeRestart("muffleWarning")
        })
    invisible(eval(parse(text = unhex(row$after_hex), keep.source = FALSE), environment))
})
box <- list(observed$value)
printed <- utils::capture.output(print(box[[1L]], useSource = FALSE))
fields <- c(row$id, row$category, "value", hex(encode(box, 1L)),
            typeof(box[[1L]]), hex(paste(class(box[[1L]]), collapse = ",")),
            mode(box[[1L]]), as.character(length(box[[1L]])),
            as.character(observed$visible), hex(paste(printed, collapse = "\n")),
            hex(paste(warnings, collapse = "\n")), hex(paste(effects, collapse = "\n")))
cat(paste(fields, collapse = "\t"), "\n", sep = "")
