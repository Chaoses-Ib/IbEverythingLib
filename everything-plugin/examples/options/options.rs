use everything_plugin::ui::winio::prelude::*;

use crate::{App, HANDLER, Mode};

pub struct MainModel {
    window: Child<Window>,
    enabled: Child<CheckBox>,
    b: Child<CheckBox>,
    s_label: Child<Label>,
    s: Child<Edit>,
    e_label: Child<Label>,
    e: Child<ComboBox>,
}

#[derive(Debug)]
pub enum MainMessage {
    Noop,
    Close,
    Redraw,
    EnabledClick,
    OptionsPage(OptionsPageMessage<App>),
}

impl From<OptionsPageMessage<App>> for MainMessage {
    fn from(value: OptionsPageMessage<App>) -> Self {
        Self::OptionsPage(value)
    }
}

impl Component for MainModel {
    type Event = ();
    type Init<'a> = OptionsPageInit<'a, App>;
    type Message = MainMessage;

    fn init(mut init: Self::Init<'_>, sender: &ComponentSender<Self>) -> Self {
        let mut window = init.window(sender);
        window.set_size(Size::new(800.0, 600.0));

        let mut enabled = Child::<CheckBox>::init(&window);
        enabled.set_text("Enable");

        let mut b = Child::<CheckBox>::init(&window);
        b.set_text("Switch");

        let mut e_label = Child::<Label>::init(&window);
        e_label.set_text("Mode:");
        let mut e = Child::<ComboBox>::init(&window);
        e.insert(0, "A");
        e.insert(1, "B");

        let mut s_label = Child::<Label>::init(&window);
        s_label.set_text("Message:");
        let mut s = Child::<Edit>::init(&window);

        HANDLER.with_app(|a| {
            let config = a.config();

            enabled.set_checked(config.enabled);
            b.set_checked(config.b);

            e.set_selection(Some(match config.e {
                Mode::A => 0,
                Mode::B => 1,
            }));

            s.set_text(&config.s);
        });

        sender.post(MainMessage::EnabledClick);

        window.show();

        Self {
            window,
            enabled,
            b,
            s_label,
            s,
            e_label,
            e,
        }
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        start! {
            sender, default: MainMessage::Noop,
            self.window => {
                WindowEvent::Close => MainMessage::Close,
                WindowEvent::Resize => MainMessage::Redraw,
            },
            self.enabled => {
                CheckBoxEvent::Click => MainMessage::EnabledClick
            }
        }
    }

    async fn update(&mut self, message: Self::Message, sender: &ComponentSender<Self>) -> bool {
        futures_util::join!(self.window.update());
        match message {
            MainMessage::Noop => false,
            MainMessage::Close => {
                sender.output(());
                false
            }
            MainMessage::Redraw => true,
            MainMessage::EnabledClick => {
                let enabled = self.enabled.is_checked();
                self.b.set_enabled(enabled);
                self.e.set_enabled(enabled);
                self.s.set_enabled(enabled);
                false
            }
            MainMessage::OptionsPage(m) => {
                tracing::debug!(?m, "Options page message");
                match m {
                    OptionsPageMessage::Save(config, tx) => {
                        config.enabled = self.enabled.is_checked();
                        config.b = self.b.is_checked();
                        config.e = match self.e.selection() {
                            Some(0) => Mode::A,
                            Some(1) => Mode::B,
                            _ => Default::default(),
                        };
                        config.s = self.s.text();
                        tx.send(config).unwrap()
                    }
                }
                false
            }
        }
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) {
        self.window.render();

        let csize = self.window.client_size();

        let m = Margin::new(5., 0., 5., 0.);
        let m_l = Margin::new(0., 5., 0., 0.);

        let mut form = layout! {
            Grid::from_str("auto,1*", "auto,auto").unwrap(),
            self.e_label => { column: 0, row: 0, margin: m_l, valign: VAlign::Center },
            self.e => { column: 1, row: 0, margin: m },
            self.s_label => { column: 0, row: 1, margin: m_l, valign: VAlign::Center },
            self.s => { column: 1, row: 1, margin: m },
        };

        let mut grid = layout! {
            Grid::from_str("auto,1*", "auto,auto,auto,1*").unwrap(),
            self.enabled => { column: 0, row: 0, margin: m },
            self.b => { column: 0, row: 1, margin: m },
            form => { column: 0, row: 2, margin: m },
        };
        grid.set_size(csize);
    }
}
