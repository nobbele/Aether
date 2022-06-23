use std::{fs::File, io::BufWriter};

use crate::{LogEntry, ENDPOINT_TYPE, LOGGER};

pub trait EndpointSuper: std::any::Any + std::fmt::Debug + std::marker::Send {}
impl<T: std::any::Any + std::fmt::Debug + std::marker::Send + std::hash::Hash> EndpointSuper for T {}

pub(crate) trait EndpointExt: EndpointSuper {
    fn endpoint_hash(&self) -> EndpointHash;
    fn fmt_message(&self, message: String) -> String;
}

impl<T: EndpointSuper + std::hash::Hash> EndpointExt for T {
    fn endpoint_hash(&self) -> EndpointHash {
        use std::hash::Hasher;

        assert_eq!(ENDPOINT_TYPE.lock().unwrap().unwrap(), self.type_id());

        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.hash(&mut hasher);
        EndpointHash(hasher.finish())
    }

    fn fmt_message(&self, message: String) -> String {
        let mut guard = LOGGER.lock().unwrap();
        let logger = guard.as_mut().expect("Uninitialized logger. Did you forget to call `aether::init` or did you drop the `KeepAlive` object early?");
        let fmt = logger
            .fmt
            .downcast_ref::<fn(LogEntry<T>) -> String>()
            .unwrap();
        fmt(LogEntry {
            time: chrono::Utc::now(),
            endpoint: self,
            text: &message,
        })
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
#[doc(hidden)]
pub struct EndpointHash(u64);

#[derive(Default)]
pub(crate) struct Endpoint {
    pub(crate) file: Option<BufWriter<File>>,
    pub(crate) silent: bool,
}
