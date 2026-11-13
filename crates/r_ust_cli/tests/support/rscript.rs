use std::path::PathBuf;
use std::process::{Command, Output};

pub(super) fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_Rscript"))
        .args(args)
        .output()
        .unwrap()
}

pub(super) fn stdout(output: Output) -> String {
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    String::from_utf8(output.stdout).unwrap()
}

pub(super) struct Script {
    pub(super) directory: PathBuf,
    pub(super) path: PathBuf,
}

impl Script {
    pub(super) fn new(source: &str) -> Self {
        static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let nonce = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "r-ust-rscript-{}-{unique}-{nonce}",
            std::process::id()
        ));
        std::fs::create_dir(&directory).unwrap();
        let path = directory.join("script with spaces.R");
        std::fs::write(&path, source).unwrap();
        Self { directory, path }
    }
}

impl Drop for Script {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
        let _ = std::fs::remove_dir(&self.directory);
    }
}
