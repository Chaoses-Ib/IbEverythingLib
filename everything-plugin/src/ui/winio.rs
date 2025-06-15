//! ## Design
//! Embedding: https://github.com/compio-rs/winio/issues/24
//!
//! Dynamic component management: https://github.com/compio-rs/winio/issues/28

use std::mem;

use futures_channel::mpsc;
use futures_util::StreamExt;
use tracing::debug;
use windows_sys::Win32::{Foundation::HWND, UI::WindowsAndMessaging::WS_OVERLAPPEDWINDOW};
use winio::{App, BorrowedWindow, Child, Component, ComponentSender, Window};

use crate::{
    PluginApp,
    ui::{OptionsPageLoadArgs, OptionsPageMessage, PageHandle},
};

pub trait OptionsPageComponent<'a>:
    Component<
        Init<'a> = OptionsPageInit<'a, Self::App>,
        Message: From<OptionsPageMessage<Self::App>>,
    > + 'static
{
    type App: PluginApp;
}

impl<'a, T, A: PluginApp> OptionsPageComponent<'a> for T
where
    T: Component<Init<'a> = OptionsPageInit<'a, A>, Message: From<OptionsPageMessage<A>>> + 'static,
{
    type App = A;
}

pub fn spawn<'a, T: OptionsPageComponent<'a>>(args: OptionsPageLoadArgs) -> PageHandle<T::App> {
    // *c_void, HWND: !Send
    let parent: usize = unsafe { mem::transmute(args.parent) };

    let (tx, rx) = mpsc::unbounded();
    let thread_handle = std::thread::spawn(move || {
        let parent: HWND = unsafe { mem::transmute(parent) };
        run::<T>(OptionsPageInit {
            parent: unsafe { BorrowedWindow::borrow_raw(parent) }.into(),
            rx: Some(rx),
        });
        // widgets::main(page_hwnd)
    });
    PageHandle { thread_handle, tx }
}

pub fn run<'a, T: OptionsPageComponent<'a>>(init: OptionsPageInit<'a, T::App>) -> T::Event {
    App::new().run::<T>(init)
}

pub struct OptionsPageInit<'a, A: PluginApp> {
    /// `MaybeBorrowedWindow`: !Clone
    parent: Option<BorrowedWindow<'a>>,

    /// Workaround for listening to external messages.
    ///
    /// A new channel is used instead of [`ComponentSender<T>`] to erase the type and keep dyn compatible.
    rx: Option<mpsc::UnboundedReceiver<OptionsPageMessage<A>>>,
}

impl<'a, A: PluginApp> From<()> for OptionsPageInit<'a, A> {
    fn from(_: ()) -> Self {
        Self {
            parent: None,
            rx: None,
        }
    }
}

impl<'a, A: PluginApp> OptionsPageInit<'a, A> {
    pub fn window<T: OptionsPageComponent<'a, App = A>>(
        &mut self,
        sender: &ComponentSender<T>,
    ) -> Child<Window> {
        let mut window = Child::<Window>::init(self.parent.clone());
        self.init(&mut window, sender);
        window
    }

    pub fn init<T: OptionsPageComponent<'a, App = A>>(
        &mut self,
        window: &mut Window,
        sender: &ComponentSender<T>,
    ) {
        if let Some(mut rx) = self.rx.take() {
            let sender = sender.clone();
            winio::compio::runtime::spawn(async move {
                while let Some(m) = rx.next().await {
                    debug!(?m, "Options page message");
                    sender.post(m.into());
                }
                debug!("Options page message channel closed");
            })
            .detach();
        }
        adjust_window(window);
    }
}

/// Adjust a window to be used in an options page.
///
/// Should be called in [`Component::init`] for the window.
pub fn adjust_window(window: &mut Window) {
    // Btw, if `window` is Window instead of &mut:
    // error[E0502]: cannot borrow `window` as immutable because it is also borrowed as mutable
    window.set_style(window.style() & !WS_OVERLAPPEDWINDOW);

    // TODO: Transparent background / background color
}
