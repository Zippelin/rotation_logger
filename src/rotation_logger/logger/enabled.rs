use std::sync::mpsc::Receiver;

use crate::rotation_logger::{Settings, logger::Message};

pub struct EnabledLogger {
    settings: Settings,
    receiver: Receiver<Message>,
}

impl EnabledLogger {
    pub fn new(settings: Settings, receiver: Receiver<Message>) -> Self {
        Self { settings, receiver }
    }

    pub fn run(&self) {
        loop {
            let msg = self.receiver.recv();

            match msg {
                Ok(val) => {
                    println!("{}", self.settings.format_message(&val))
                }
                Err(err) => {
                    println!("Channel closed. Error: {err}")
                }
            }
        }
    }
}
