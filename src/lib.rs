use crate::logger::Logger;
use std::{collections::HashMap, fs::File, hash::Hash, io::BufWriter, sync::Mutex};

mod builders;
mod logger;

pub use builders::{EndpointBuilder, LoggerBuilder};
pub use logger::KeepAlive;

lazy_static::lazy_static! {
    static ref ENDPOINT_TYPE: Mutex<Option<std::any::TypeId>> = Mutex::new(None);
    static ref LOGGER: Mutex<Option<Logger>> = Mutex::new(None);
}

pub trait EndpointSuper: std::any::Any + std::fmt::Debug + std::hash::Hash {}
impl<T: std::any::Any + std::fmt::Debug + std::hash::Hash> EndpointSuper for T {}

#[macro_export]
macro_rules! log {
    ($target:expr, $($arg:tt)*) => {
        aether::impl_log($target, format!($($arg)*))
    };
}

#[doc(hidden)]
pub fn impl_log<EP: EndpointSuper + 'static>(endpoint: EP, message: String) {
    let mut guard = LOGGER.lock().unwrap();
    let logger = guard.as_mut().expect("Uninitialized logger. Did you forget to call `aether::init` or did you drop the `KeepAlive` object early?");
    let fmt = logger
        .fmt
        .downcast_ref::<fn(LogEntry<EP>) -> String>()
        .unwrap();
    let output = fmt(LogEntry {
        time: chrono::Utc::now(),
        endpoint: &endpoint,
        text: &message,
    });

    logger.log(hash(&endpoint), output);
}

pub struct LogEntry<'a, EP> {
    pub time: chrono::DateTime<chrono::Utc>,
    pub endpoint: &'a EP,
    pub text: &'a str,
}

#[derive(Hash, PartialEq, Eq)]
struct EndpointHash(u64);

fn hash(obj: &impl EndpointSuper) -> EndpointHash {
    use std::hash::Hasher;

    assert_eq!(ENDPOINT_TYPE.lock().unwrap().unwrap(), obj.type_id());

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    obj.hash(&mut hasher);
    EndpointHash(hasher.finish())
}

#[derive(Default)]
struct Endpoint {
    file: Option<BufWriter<File>>,
    silent: bool,
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
