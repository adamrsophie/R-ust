use std::process::Command;

#[test]
fn stop_output_order_preserves_emitted_output_in_both_entrypoints() {
    for (binary, flags) in [
        (
            env!("CARGO_BIN_EXE_r_ust_cli"),
            vec!["--vanilla", "--no-echo", "-e"],
        ),
        (env!("CARGO_BIN_EXE_Rscript"), vec!["--vanilla", "-e"]),
    ] {
        let result = Command::new(binary)
            .args(flags)
            .arg("cat('BEFORE');stop('boom');cat('AFTER')")
            .output()
            .unwrap();
        assert_eq!(result.status.code(), Some(1), "{binary}");
        assert_eq!(result.stdout, b"BEFORE", "{binary}");
        assert_eq!(result.stderr, b"boom\n", "{binary}");
    }
}
