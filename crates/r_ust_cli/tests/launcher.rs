use std::process::Command;

#[test]
fn expressions_use_unix_launcher_escapes_in_both_frontends() {
    for binary in [
        env!("CARGO_BIN_EXE_r_ust_cli"),
        env!("CARGO_BIN_EXE_Rscript"),
    ] {
        for source in [r"cat('a\0101b')", r"cat('a\x41b')"] {
            let output = Command::new(binary)
                .args(["--no-echo", "-e", source])
                .output()
                .unwrap();
            assert!(output.status.success(), "{binary}: {output:?}");
            assert_eq!(output.stdout, b"aAb", "{binary}: {source}");
            assert!(output.stderr.is_empty(), "{output:?}");
        }
        let output = Command::new(binary)
            .args([
                "--no-echo",
                "-e",
                "a<-commandArgs(FALSE);writeLines(a[length(a)]);q('no')",
                "-e",
                r"cat('a\nb')",
            ])
            .output()
            .unwrap();
        assert!(output.status.success(), "{binary}: {output:?}");
        assert_eq!(output.stdout, b"cat('a~n~b')\n", "{binary}");
        assert!(output.stderr.is_empty(), "{output:?}");
    }
}
