#[macro_export]
macro_rules! plugin_main {
    ($app_type:ty, $handler_builder:expr) => {
        static HANDLER: ::std::sync::LazyLock<::everything_plugin::PluginHandler<$app_type>> =
            ::std::sync::LazyLock::new(|| $handler_builder);

        /// - `msg` is a `EVERYTHING_PLUGIN_PM_*` message.
        /// - `data` will depend on the `EVERYTHING_PLUGIN_PM_*` message.
        ///
        /// return data based on the `EVERYTHING_PLUGIN_PM_*` message.
        #[unsafe(no_mangle)]
        pub extern "system" fn everything_plugin_proc(
            msg: u32,
            data: *mut ::std::ffi::c_void,
        ) -> *mut ::std::ffi::c_void {
            HANDLER.handle(msg, data)
        }
    };
}
