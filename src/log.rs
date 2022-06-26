use crate::{endpoint::EndpointExt, EndpointSuper, LOGGER};

/// Log entry.
pub struct LogEntry<'a, EP> {
    /// The time the log was recorded. See [`chrono::DateTime`] for more details.
    pub time: chrono::DateTime<chrono::Utc>,
    /// Specifies which endpoint this log was reported to.
    pub endpoint: &'a EP,
    /// The message that was logged.
    pub text: &'a str,
}

#[doc(hidden)]
pub fn impl_log<EP: EndpointSuper + std::hash::Hash + 'static>(endpoint: EP, message: String) {
    let output = endpoint.fmt_message(message);
    let mut guard = LOGGER.lock().unwrap();
    let logger = guard.as_mut().expect("Uninitialized logger. Did you forget to call `aether::init` or did you drop the `KeepAlive` object early?");
    logger.log(endpoint.endpoint_hash(), output);
}
