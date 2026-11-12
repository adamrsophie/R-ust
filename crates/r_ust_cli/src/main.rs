fn main() {
    match r_ust_cli::run_main(std::env::args()) {
        Ok(0) => {}
        Ok(status) => std::process::exit(status),
        Err(message) => {
            eprintln!("{message}");
            std::process::exit(1);
        }
    }
}
