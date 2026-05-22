mod config;

fn main() {
    // Initialize env_logger to support standard verbose log outputs.
    // If RUST_LOG environment variable is not defined, we fallback to "debug"
    // to satisfy the user's preference for verbose (DEBUG) development logs.
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "debug");
    }
    env_logger::init();

    log::info!("Starting Transcriber RUST application...");

    // Bootstrap and parse the local configuration settings
    let config = config::Config::load();
    log::debug!("Loaded configuration: {:?}", config);

    log::info!("Transcriber RUST bootstrapper finished successfully.");
}
