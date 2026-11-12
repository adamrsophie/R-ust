fn main() {
    let mut args = std::env::args();
    let program = args.next().unwrap_or_else(|| "r_ust_cli".to_string());
    let mut config = r_ust_cli::CliConfig::from_args(args).unwrap_or_else(|message| {
        eprintln!("{message}");
        std::process::exit(2);
    });
    config.command_args[0] = program;
    match r_ust_cli::run(config) {
        Ok(0) => {}
        Ok(status) => std::process::exit(status),
        Err(message) => {
            eprintln!("{message}");
            std::process::exit(1);
        }
    }
}
