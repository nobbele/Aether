use crate::logger::Logger;
use std::{collections::HashMap, sync::Mutex};

mod builders;
mod endpoint;
mod log;
mod logger;
mod scoped;

pub use builders::{EndpointBuilder, LoggerBuilder};
pub use endpoint::EndpointSuper;
pub use log::{impl_log, LogEntry};
pub use logger::KeepAlive;
pub use scoped::{impl_slog, scoped};

lazy_static::lazy_static! {
    static ref ENDPOINT_TYPE: Mutex<Option<std::any::TypeId>> = Mutex::new(None);
    static ref LOGGER: Mutex<Option<Logger>> = Mutex::new(None);
}

#[macro_export]
macro_rules! slog {
    ($($arg:tt)*) => {
        aether::impl_slog(format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log {
    ($target:expr, $($arg:tt)*) => {
        aether::impl_log($target, format!($($arg)*))
    };
}

pub fn init<EP: EndpointSuper>() -> LoggerBuilder<EP> {
    let mut guard = ENDPOINT_TYPE.lock().unwrap();
    if guard.replace(std::any::TypeId::of::<EP>()).is_some() {
        panic!("Logger has already been (at least partially) initialized!");
    }

    LoggerBuilder {
        base_path: None,
        fmt: |log| {
            format!(
                "{} [{:?}] {}",
                log.time.format("%T.%3f"),
                log.endpoint,
                log.text
            )
        },
        endpoints: HashMap::new(),
    }
}
