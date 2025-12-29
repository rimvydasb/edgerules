fn main() {
    if let Err(error) = edge_rules_cli::run() {
        eprintln!("{}", error);
        std::process::exit(1);
    }
}
