/// ```ignore
/// plugin_main!(App, {
///     PluginHandler::builder()
///         .name("Test Plugin")
///         .description("A test plugin for Everything")
///         .author("Chaoses-Ib")
///         .version("0.1.0")
///         .link("https://github.com/Chaoses-Ib/IbEverythingLib")
///         .options_pages(vec![
///             OptionsPage::builder()
///                 .name("Test Plugin")
///                 .load(ui::winio::spawn::<options::MainModel>)
///                 .build(),
///         ])
///         .build()
/// });
/// ```
#[macro_export]
macro_rules! plugin_main {
    ($app_type:ty, $handler_builder:expr) => {
        static HANDLER: ::std::sync::LazyLock<::everything_plugin::PluginHandler<$app_type>> =
            ::std::sync::LazyLock::new(|| {
                ::everything_plugin::PluginHandler::<$app_type>::handle_init_i18n(
                    ::everything_plugin::sys::EVERYTHING_PLUGIN_PM_INIT,
                    0 as _,
                );

                $handler_builder
            });

        /// - `msg` is a `EVERYTHING_PLUGIN_PM_*` message.
        /// - `data` will depend on the `EVERYTHING_PLUGIN_PM_*` message.
        ///
        /// return data based on the `EVERYTHING_PLUGIN_PM_*` message.
        #[unsafe(no_mangle)]
        pub extern "system" fn everything_plugin_proc(
            msg: u32,
            data: *mut ::std::ffi::c_void,
        ) -> *mut ::std::ffi::c_void {
            ::everything_plugin::PluginHandler::<$app_type>::handle_init_i18n(msg, data);

            HANDLER.handle(msg, data)
        }
    };
}
