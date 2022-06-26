use crate::{
    endpoint::{EndpointExt, EndpointHash},
    logger::{setup_logger, KeepAlive},
    EndpointSuper, LogEntry,
};
use std::{
    collections::HashMap,
    marker::PhantomData,
    path::{Path, PathBuf},
};

/// Builder for an endpoint.
pub struct EndpointBuilder {
    pub(crate) path: Option<PathBuf>,
    pub(crate) silent: bool,
    pub(crate) disabled: bool,
}

impl EndpointBuilder {
    /// Disables file output. This is the default.
    pub fn no_path(mut self) -> Self {
        self.path = None;
        self
    }

    /// Enables file output.
    pub fn path(mut self, path: impl AsRef<Path>) -> Self {
        self.path = Some(path.as_ref().into());
        self
    }

    /// Don't log to console.
    pub fn silent(mut self) -> Self {
        self.silent = true;
        self
    }

    /// Ignore this endpoint.
    pub fn disable(mut self) -> Self {
        self.disabled = true;
        self
    }
}

/// Builder for the logger type.
pub struct LoggerBuilder<EP: EndpointSuper> {
    pub(crate) base_path: Option<PathBuf>,
    pub(crate) fmt: fn(LogEntry<EP>) -> String,
    pub(crate) endpoints: HashMap<EndpointHash, EndpointBuilder>,
}

impl<EP: EndpointSuper + std::hash::Hash> LoggerBuilder<EP> {
    /// Sets the format for all log entries.
    pub fn format(mut self, fmt: fn(LogEntry<EP>) -> String) -> Self {
        self.fmt = fmt;
        self
    }

    /// Specifies the base path for files and archives.
    pub fn base_path(mut self, path: impl AsRef<Path>) -> Self {
        self.base_path = Some(path.as_ref().into());
        self
    }

    /// Setup an endpoint.
    pub fn setup(mut self, endpoint: EP, setup: fn(EndpointBuilder) -> EndpointBuilder) -> Self {
        self.endpoints.insert(
            endpoint.endpoint_hash(),
            setup(EndpointBuilder {
                path: None,
                silent: false,
                disabled: false,
            }),
        );
        self
    }

    /// Construct the logger. See [`KeepAlive`] for more details.
    pub fn build(self) -> KeepAlive {
        setup_logger(self);
        KeepAlive(PhantomData)
    }
}
