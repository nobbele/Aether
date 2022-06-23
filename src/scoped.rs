use std::sync::Mutex;

use crate::{endpoint::EndpointExt, EndpointSuper, LOGGER};

lazy_static::lazy_static! {
    pub(crate) static ref SCOPE: Mutex<Option<Box<dyn EndpointExt>>> = Mutex::new(None);
}

#[doc(hidden)]
pub fn impl_slog(message: String) {
    let scope_guard = SCOPE.lock().unwrap();
    let scope = scope_guard
        .as_ref()
        .expect("Tried using scoped log outside of a log scope.");

    let output = scope.fmt_message(message);

    let mut guard = LOGGER.lock().unwrap();
    let logger = guard.as_mut().expect("Uninitialized logger. Did you forget to call `aether::init` or did you drop the `KeepAlive` object early?");
    logger.log(scope.endpoint_hash(), output);
}

pub fn scoped<EP: EndpointSuper + std::hash::Hash>(endpoint: EP, f: impl FnOnce()) {
    let mut scope_guard = SCOPE.lock().unwrap();
    let prev_scope = scope_guard.replace(Box::new(endpoint));
    std::mem::drop(scope_guard);
    f();
    let mut scope_guard = SCOPE.lock().unwrap();
    *scope_guard = prev_scope;
}
