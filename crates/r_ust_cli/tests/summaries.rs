use std::process::{Command, Output};

const BINARIES: [&str; 2] = [
    env!("CARGO_BIN_EXE_r_ust_cli"),
    env!("CARGO_BIN_EXE_Rscript"),
];

fn run(binary: &str, source: &str) -> Output {
    Command::new(binary)
        .args(["--vanilla", "--no-echo", "-e", source])
        .env("LC_ALL", "en_US.UTF-8")
        .output()
        .unwrap()
}

#[test]
fn summaries_print_once_through_direct_and_forwarded_calls() {
    let source =
        "sum(1L,2L);prod(2L,3L);f<-function(...)sum(...);f(1L,2L);apply(matrix(1:4,nrow=2),2,sum)";
    for binary in BINARIES {
        let output = run(binary, source);
        assert_eq!(output.status.code(), Some(0), "{binary}; {output:?}");
        assert_eq!(output.stdout, b"[1] 3\n[1] 6\n[1] 3\n[1] 3 7\n");
        assert!(output.stderr.is_empty(), "{binary}; {output:?}");
    }
}

#[test]
fn summary_errors_preserve_prior_output_and_stop_both_frontends() {
    for binary in BINARIES {
        for (source, stdout, message) in [
            (
                "sum(,{cat('LATE\\n');1});cat('AFTER\\n')",
                "",
                "argument error: argument 1 is empty\n",
            ),
            (
                "cat('BEFORE\\n');prod(1,na.rm=NULL);cat('AFTER\\n')",
                "BEFORE\n",
                "domain error: NA in coercion to boolean\n",
            ),
            (
                "sum({cat('DATA\\n');1},na.rm=TRUE,na.rm={cat('FLAG\\n');FALSE});cat('AFTER\\n')",
                "DATA\nFLAG\n",
                "argument error: formal argument \"na.rm\" matched by multiple actual arguments\n",
            ),
            (
                "cat('BEFORE\\n');sum(factor('a'),na.rm=NA);cat('AFTER\\n')",
                "BEFORE\n",
                "type error: 'sum' not meaningful for factors\n",
            ),
        ] {
            let output = run(binary, source);
            assert_eq!(output.status.code(), Some(1), "{binary}; {source}");
            assert_eq!(output.stdout, stdout.as_bytes(), "{binary}; {source}");
            assert_eq!(output.stderr, message.as_bytes(), "{binary}; {source}");
        }
    }
}

#[test]
fn eager_summary_warnings_keep_their_position_in_merged_output() {
    let source = "options(warn=1);sum({cat('FIRST\\n');1},as.integer('bad'),{cat('LAST\\n');2},na.rm=TRUE);cat('AFTER\\n')";
    for binary in BINARIES {
        let output = Command::new("/bin/sh")
            .args([
                "-c",
                "exec \"$1\" --vanilla --no-echo -e \"$2\" 2>&1",
                "summaries",
                binary,
                source,
            ])
            .env("LC_ALL", "en_US.UTF-8")
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(0), "{binary}; {output:?}");
        assert_eq!(
            output.stdout, b"FIRST\nWarning: NAs introduced by coercion\nLAST\n[1] 3\nAFTER\n",
            "{binary}"
        );
        assert!(output.stderr.is_empty(), "{binary}; {output:?}");
    }
}
