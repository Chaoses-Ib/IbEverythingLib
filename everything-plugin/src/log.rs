/// A convenient function to initialize [`tracing`] with a default configuration.
#[cfg(feature = "tracing")]
pub fn tracing_init() {
    tracing_subscriber::fmt()
        // TODO: Non-block?
        .with_writer(anstream::stderr)
        .with_max_level(tracing::Level::DEBUG)
        .init();

    #[cfg(debug_assertions)]
    std::panic::set_hook(Box::new(|info| {
        tracing_panic::panic_hook(info);
        std::thread::sleep(std::time::Duration::from_secs(60));
    }));
}
