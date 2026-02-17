use std::{
    ptr,
    sync::{
        atomic::{AtomicPtr, Ordering},
        mpsc::{Sender, channel},
    },
    thread::{self, JoinHandle},
};

use crate::rotation_logger::Settings;

mod enabled;
mod message;

pub use enabled::EnabledLogger;

pub use message::Message;

pub static LOG_SENDER: AtomicPtr<Sender<Message>> = AtomicPtr::new(ptr::null_mut());

#[derive(Clone)]
pub enum Logger {
    Enabled(Settings),
    Disabled,
}

impl Logger {
    pub fn new(settings: Settings) -> Self {
        if settings.is_enabled() {
            Self::enabled(settings)
        } else {
            Self::disabled()
        }
    }
    pub fn enabled(settings: Settings) -> Self {
        Self::Enabled(settings)
    }

    pub fn disabled() -> Self {
        Self::Disabled
    }

    pub fn log(&self, modules: &Vec<String>, text: &str) {
        match &self {
            Logger::Enabled(_) => {
                let prt = LOG_SENDER.load(Ordering::Acquire);

                if !prt.is_null() {
                    unsafe {
                        let sender = &*prt;
                        let message = Message::new(modules, text);
                        let _ = sender.send(message);
                    }
                }
            }
            Logger::Disabled => return,
        }
    }

    pub fn run_async(&self) -> Option<JoinHandle<()>> {
        match self {
            Logger::Enabled(settings) => {
                let (tx, rx) = channel::<Message>();

                let boxed = Box::new(tx.clone());
                let ptr = Box::into_raw(boxed);

                LOG_SENDER.store(ptr, Ordering::Relaxed);
                let logger = EnabledLogger::new(settings.clone(), rx);

                Some(thread::spawn(move || logger.run()))
            }
            Logger::Disabled => None,
        }
    }
}
