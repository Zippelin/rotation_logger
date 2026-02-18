//! # Macros for logger.
//!
//! Possible to log data as ident test or strings
//!
//! Example as simple log:
//!
//! ```
//! log!("Some important data.");
//! ```
//!
//! Example as logger with modules text:
//! Text modules must be surrounded by `[..]` brackets.
//! ```
//! log!(
//!     ["MODULE_01", "MODULE_02"],
//!     "Some important data."
//! );
//! ```
//!
//! Example as logger with modules ident:
//! Ident modules must be surrounded by `(..)` brackets.
//! ```
//! log!((RAW_MODULE, RAW_MODULE2, RAW_MODULE3), "some");
//! ```
//!

/// Thread safe macros to log messages.
#[macro_export]
macro_rules! log {
    ([$($modules:expr),*], $message:expr) => {
        let prt = rotation_logger::LOG_SENDER.load(std::sync::atomic::Ordering::Acquire);
        let modules = vec![$($modules.to_string()),+];
        if !prt.is_null() {
            unsafe {
                let sender = &*prt;
                let message = rotation_logger::Message::new(&modules, $message);
                let _ = sender.send(message);
            }
        }
    };
    (($($modules:ident),*), $message:expr) => {{
        let prt = rotation_logger::LOG_SENDER.load(std::sync::atomic::Ordering::Acquire);
        if !prt.is_null() {
            unsafe {
                let sender = &*prt;
                let modules = vec![$(stringify!($modules).to_string()),*];
                let message = rotation_logger::Message::new(&modules, $message);
                let _ = sender.send(message);
            }
        }
    }};
    ($message:expr) => {
        let prt = rotation_logger::LOG_SENDER.load(std::sync::atomic::Ordering::Acquire);
        if !prt.is_null() {
            unsafe {
                let sender = &*prt;
                let message = rotation_logger::Message::new(&vec![], $message);
                let _ = sender.send(message);
            }
        }
    };
}
