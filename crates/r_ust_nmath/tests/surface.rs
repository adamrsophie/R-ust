use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

#[test]
fn rust_public_surface_matches_standalone_rmath_header() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let header = manifest_dir.join("../..").join("tests/oracle/Rmath.h0.in");
    let header = fs::read_to_string(header).expect("Rmath.h0.in source snapshot");

    let expected = parse_header_functions(&header);
    let actual: BTreeSet<_> = r_ust_nmath::PUBLIC_FUNCTIONS.iter().copied().collect();

    let missing: Vec<_> = expected.difference(&actual).copied().collect();
    let extra: Vec<_> = actual.difference(&expected).copied().collect();

    assert!(missing.is_empty(), "missing public functions: {missing:?}");
    assert!(extra.is_empty(), "extra public functions: {extra:?}");
}

fn parse_header_functions(header: &str) -> BTreeSet<&str> {
    let mut names = BTreeSet::new();
    let mut skip_rcpp_compat = false;

    for line in header.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("#if defined(COMPILING_RCPP)") {
            skip_rcpp_compat = true;
            continue;
        }
        if skip_rcpp_compat {
            if trimmed.starts_with("/* Non-central F Distribution */") {
                skip_rcpp_compat = false;
            }
            continue;
        }

        for prefix in ["double", "int", "void"] {
            let Some(rest) = trimmed.strip_prefix(prefix) else {
                continue;
            };
            let rest = rest.trim_start();
            let Some(paren) = rest.find('(') else {
                continue;
            };
            let name = rest[..paren].trim();
            if name.is_empty() || name.contains('*') || name.contains(' ') {
                continue;
            }
            names.insert(name);
        }
    }

    names
}
