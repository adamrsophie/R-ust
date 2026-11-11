fn main() {
    let config =
        r_ust_cli::CliConfig::from_args(std::env::args().skip(1)).unwrap_or_else(|message| {
            eprintln!("{message}");
            std::process::exit(2);
        });
    if let Err(message) = r_ust_cli::run(config) {
        eprintln!("{message}");
        std::process::exit(2);
    }
}
