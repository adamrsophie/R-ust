use std::io::Write;
use std::process::{Command, Stdio};

#[test]
fn batch_readline_does_not_consume_following_source_lines() {
    for executable in [
        env!("CARGO_BIN_EXE_r_ust_cli"),
        env!("CARGO_BIN_EXE_Rscript"),
    ] {
        let mut child = Command::new(executable)
            .arg("-")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(b"a<-readline(\"ask: \");cat(\"ANSWER:\",a,\"\\n\",sep=\"\")\nprint(7)\n")
            .unwrap();
        let output = child.wait_with_output().unwrap();
        assert!(output.status.success(), "{executable}: {output:?}");
        assert_eq!(
            String::from_utf8(output.stdout).unwrap(),
            "ask: \nANSWER:\n[1] 7\n"
        );
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn expression_readline_remains_non_interactive() {
    for executable in [
        env!("CARGO_BIN_EXE_r_ust_cli"),
        env!("CARGO_BIN_EXE_Rscript"),
    ] {
        let output = Command::new(executable)
            .args(["-e", "withVisible(readline(\"p\"))$visible"])
            .stdin(Stdio::null())
            .output()
            .unwrap();
        assert!(output.status.success(), "{executable}: {output:?}");
        assert_eq!(String::from_utf8(output.stdout).unwrap(), "p\n[1] TRUE\n");
        assert!(output.stderr.is_empty());
    }
}
