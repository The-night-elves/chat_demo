use crate::client_message::Message;
use crate::{ClientMessage, JoinRoom, Login, SendMessage, ServerMessage};
use fltk::{app, group::Flex, prelude::*, window, *};
use fltk_table::{SmartTable, TableOpts};
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;
use tracing::info;

pub struct View {
    pub width: i32,
    pub height: i32,
    pub title: String,
    pub tx: mpsc::Sender<ClientMessage>,
    pub subscribe_topic: Arc<RwLock<String>>,
}

impl View {
    pub fn new(width: i32, height: i32, title: String, tx: mpsc::Sender<ClientMessage>) -> Self {
        Self {
            width,
            height,
            title,
            tx,
            subscribe_topic: Arc::new(RwLock::new("".into())),
        }
    }

    // message dispatch
    fn message_dispatch(tx: mpsc::Sender<ClientMessage>, msg: ClientMessage) {
        tokio::task::spawn_blocking(move || {
            info!("send message: {msg:?}");
            tx.blocking_send(msg)?;
            Ok::<(), anyhow::Error>(())
        });
    }

    fn header(&self) -> Flex {
        let header = Flex::default()
            .with_size(&self.width - 25, 30)
            .row()
            .with_pos(10, 10);

        frame::Frame::default()
            .with_size(5, 5)
            .with_label("user name");
        let input = input::Input::default().with_size(5, 5);

        let mut btn = button::Button::default()
            .with_size(5, 5)
            .with_label("login");

        let tx = self.tx.clone();
        btn.set_callback(move |_e| {
            let val = input.value();
            if !val.is_empty() {
                Self::message_dispatch(
                    tx.clone(),
                    ClientMessage {
                        topic: "".to_string(),
                        message: Some(Message::Login(Login { name: val })),
                    },
                );
            }
        });
        header
    }

    fn subscription(&self) -> Flex {
        let subs = Flex::default()
            .with_size(&self.width - 25, 30)
            .with_pos(10, 50);

        frame::Frame::default().with_size(5, 5).with_label("订阅");

        let input = input::Input::default().with_size(5, 5);

        let mut btn = button::Button::default()
            .with_size(5, 5)
            .with_label("subscription");

        let tx = self.tx.clone();
        let subscribe_topic = self.subscribe_topic.clone();
        btn.set_callback(move |_| {
            let val = input.value();
            if !val.is_empty() {
                *subscribe_topic.write().unwrap() = val.clone().into();
                Self::message_dispatch(
                    tx.clone(),
                    ClientMessage {
                        topic: val,
                        message: Some(Message::JoinRoom(JoinRoom {})),
                    },
                );
            }
        });

        subs
    }

    fn send_message(&self) -> Flex {
        let msg = Flex::default()
            .with_size(&self.width - 25, 30)
            .with_pos(10, 90);

        frame::Frame::default()
            .with_size(5, 0)
            .with_label("input message");

        let mut input = input::Input::default().with_size(9, 0);

        let mut btn = button::Button::default()
            .with_size(0, 0)
            .with_label("submit");

        let tx = self.tx.clone();
        let subscribe_topic = self.subscribe_topic.clone();
        btn.set_callback(move |_| {
            let topic = &*subscribe_topic.read().unwrap();
            info!("subscribe_topic {:?}", topic);
            let val = input.value();
            if !val.is_empty() {
                Self::message_dispatch(
                    tx.clone(),
                    ClientMessage {
                        topic: topic.to_string(),
                        message: Some(SendMessage(val)),
                    },
                );
                input.set_value("");
            }
        });

        msg
    }

    pub fn show(&mut self, mut rx: mpsc::Receiver<ServerMessage>) {
        let app = app::App::default();
        let mut wind = window::Window::default()
            .with_size(self.width, self.height)
            .with_label(self.title.as_str())
            .center_screen();
        // 可以修改窗口的大小
        wind.make_resizable(true);

        self.header().end();
        self.subscription().end();
        self.send_message().end();

        let mut table = SmartTable::default()
            .with_size(&self.width - 25, &self.height - 140)
            .with_opts(TableOpts {
                rows: 1,
                cols: 3,
                ..Default::default()
            })
            .with_pos(10, 180);

        let col_headers = ["topic", "sequence", "message"];
        table.set_col_header_value(0, "topic");
        col_headers.iter().enumerate().for_each(|(i, v)| {
            table.set_col_header_value(i as i32, v);
        });

        table.end();

        // wind 中间的东西会加入 wind
        wind.end();
        // wind builder
        wind.show();

        tokio::task::spawn_blocking(move || {
            let mut first = true;
            while let Some(msg) = rx.blocking_recv() {
                info!("recv {:?}", msg.topic);
                match msg.message {
                    None => {}
                    Some(data) => {
                        let seq = msg.sequence.to_string();
                        let row = &vec![msg.topic.as_str(), seq.as_str(), data.as_str()];
                        table.append_row("", row);
                        if first {
                            table.remove_row(0);
                            first = false;
                        }
                    }
                }
                wind.redraw();
            }
        });

        app.run().unwrap();
    }
}
