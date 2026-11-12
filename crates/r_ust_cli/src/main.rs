fn main() {
    let config =
        r_ust_cli::CliConfig::from_args(std::env::args().skip(1)).unwrap_or_else(|message| {
            eprintln!("{message}");
            std::process::exit(2);
        });
    match r_ust_cli::run(config) {
        Ok(0) => {}
        Ok(status) => std::process::exit(status),
        Err(message) => {
            eprintln!("{message}");
            std::process::exit(1);
        }
    }
}
