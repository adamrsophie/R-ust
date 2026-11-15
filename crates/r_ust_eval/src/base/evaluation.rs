// R 4.6.0 src/library/base/R/funprog.R: Copyright (C) 1995-2014 The R Core Team
// R 4.6.0 src/library/base/R/eval.R: Copyright (C) 1995-2017 The R Core Team
// R 4.6.0 src/library/base/R/assign.R: Copyright (C) 1995-2012 The R Core Team
// R 4.6.0 src/library/base/R/identical.R: Copyright (C) 1995-2017 The R Core Team
// Part of the R package, https://www.R-project.org
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; either version 2 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// A copy of the GNU General Public License is available at
// https://www.R-project.org/Licenses/

pub(super) const DEFINITIONS: &[(&str, &str)] = &[
    ("identity", "function(x) x"),
    ("force", "function(x) x"),
    (
        "eval",
        r#"function(expr, envir = parent.frame(),
        enclos = if(is.list(envir) || is.pairlist(envir)) parent.frame() else baseenv())
        .Internal(eval(expr, envir, enclos))"#,
    ),
    (
        "evalq",
        r#"function(expr, envir = parent.frame(),
        enclos = if(is.list(envir) || is.pairlist(envir)) parent.frame() else baseenv())
        .Internal(eval(substitute(expr), envir, enclos))"#,
    ),
    (
        "eval.parent",
        r#"function(expr, n = 1) {
        p <- parent.frame(n + 1)
        eval(expr, p)
    }"#,
    ),
    (
        "assign",
        r#"function(x, value, pos = -1, envir = as.environment(pos),
        inherits = FALSE, immediate = TRUE)
        .Internal(assign(x, value, envir, inherits))"#,
    ),
    (
        "identical",
        r#"function(x, y, num.eq = TRUE, single.NA = TRUE,
                      attrib.as.set = TRUE, ignore.bytecode = TRUE,
                      ignore.environment = FALSE, ignore.srcref = TRUE,
                      extptr.as.ref = FALSE)
    .Internal(identical(x,y, num.eq, single.NA, attrib.as.set,
                        ignore.bytecode, ignore.environment, ignore.srcref,
                        extptr.as.ref))"#,
    ),
];
