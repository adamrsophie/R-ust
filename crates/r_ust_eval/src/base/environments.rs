// R 4.6.0 src/library/base/R/eval.R: Copyright (C) 1995-2017 The R Core Team
// R 4.6.0 src/library/base/R/environment.R and sys.R: Copyright (C) 1995-2012 The R Core Team
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
    ("environment", "function(fun = NULL) .Internal(environment(fun))"),
    ("new.env", "function(hash = TRUE, parent = parent.frame(), size = 29L) .Internal(new.env(hash, parent, size))"),
    ("parent.frame", "function(n = 1) .Internal(parent.frame(n))"),
    ("sys.frame", "function(which = 0L) .Internal(sys.frame(which))"),
    ("sys.function", "function(which = 0L) .Internal(sys.function(which))"),
    ("sys.parent", "function(n = 1L) .Internal(sys.parent(n))"),
    ("sys.nframe", "function() .Internal(sys.nframe())"),
    ("parent.env", "function(env) .Internal(parent.env(env))"),
    (
        "environmentName",
        "function(env) .Internal(environmentName(env))",
    ),
];
