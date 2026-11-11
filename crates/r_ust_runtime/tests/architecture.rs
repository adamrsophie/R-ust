use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn workspace_uses_rust_owned_runtime_crates() {
    let root = workspace_root();
    let manifest = read(root.join("Cargo.toml"));

    for crate_name in [
        "r_ust_cli",
        "r_ust_eval",
        "r_ust_ffi",
        "r_ust_gc",
        "r_ust_nmath",
        "r_ust_parse",
        "r_ust_runtime",
        "r_ust_sexp",
    ] {
        assert!(
            manifest.contains(&format!("\"crates/{crate_name}\"")),
            "workspace is missing {crate_name}"
        );
    }
    assert!(
        !manifest.contains("r_ust_nmath_capi"),
        "deleted C-backed nmath crate must not return to the workspace"
    );
}

#[test]
fn runtime_crates_do_not_compile_upstream_c_sources() {
    let root = workspace_root();
    for entry in fs::read_dir(root.join("crates")).expect("crates directory") {
        let entry = entry.expect("crate entry");
        if !entry.file_type().expect("crate file type").is_dir() {
            continue;
        }
        let crate_dir = entry.path();
        let manifest_path = crate_dir.join("Cargo.toml");
        if manifest_path.exists() {
            let manifest = read(&manifest_path);
            assert_no_cc_dependency(&manifest, &manifest_path);
            assert!(
                !manifest.contains("r_ust_nmath_capi"),
                "{} must not depend on the removed C-backed crate",
                manifest_path.display()
            );
        }

        let build_rs = crate_dir.join("build.rs");
        if build_rs.exists() {
            let script = read(&build_rs);
            assert!(
                !script.contains("upstream/R-4.6.0/src"),
                "{} must not compile upstream R implementation sources",
                build_rs.display()
            );
        }
    }
}

#[test]
fn runtime_component_registry_matches_workspace_shape() {
    let mut actual: Vec<_> = r_ust_runtime::component_names().collect();
    actual.sort_unstable();
    assert_eq!(
        actual,
        vec![
            "r_ust_cli",
            "r_ust_eval",
            "r_ust_ffi",
            "r_ust_gc",
            "r_ust_nmath",
            "r_ust_parse",
            "r_ust_runtime",
            "r_ust_sexp",
        ]
    );
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root")
}

fn read(path: impl AsRef<Path>) -> String {
    fs::read_to_string(path.as_ref())
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.as_ref().display()))
}

fn assert_no_cc_dependency(manifest: &str, path: &Path) {
    for line in manifest.lines() {
        let line = line.split('#').next().unwrap_or("").trim();
        if line.starts_with("cc") && line[2..].trim_start().starts_with('=') {
            panic!(
                "{} must not depend on cc to compile R C sources",
                path.display()
            );
        }
    }
}
