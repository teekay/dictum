fn main() {
    let sqlite = std::env::var("CARGO_FEATURE_SQLITE").is_ok();
    let grafeo = std::env::var("CARGO_FEATURE_GRAFEO").is_ok();
    match (sqlite, grafeo) {
        (true, true) => panic!(
            "Features 'sqlite' and 'grafeo' are mutually exclusive. Enable exactly one."
        ),
        (false, false) => panic!(
            "No storage backend selected. Enable either 'sqlite' (default) or 'grafeo'."
        ),
        _ => {}
    }
}
