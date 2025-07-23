#[cfg(feature = "tracing")]
pub use tracing::{debug, error, info, trace, warn};

/// A convenient function to initialize [`tracing`] with a default configuration.
#[cfg(feature = "tracing")]
pub fn tracing_init() {
    #[cfg(not(feature = "tracing-appender"))]
    let stderr = anstream::stderr;
    #[cfg(feature = "tracing-appender")]
    let stderr = {
        let (non_blocking, guard) = tracing_appender::non_blocking(anstream::stderr());
        std::mem::forget(guard);
        non_blocking
    };

    tracing_subscriber::fmt()
        .with_writer(stderr)
        .with_max_level(tracing::Level::DEBUG)
        .init();

    #[cfg(debug_assertions)]
    std::panic::set_hook(Box::new(|info| {
        tracing_panic::panic_hook(info);
        std::thread::sleep(std::time::Duration::from_secs(60));
    }));
}
