// R 4.6.0 src/library/base/R/formals.R: Copyright (C) 1995-2023 The R Core Team
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

pub(super) const DEFINITIONS: &[(&str, &str)] = &[(
    "formals",
    r#"function(fun = sys.function(sys.parent()), envir = parent.frame()) {
        if(is.character(fun))
            fun <- get(fun, mode = "function", envir = envir)
        .Internal(formals(fun))
    }"#,
)];
