use crate::{EndpointHash, EndpointSuper, KeepAlive, LogEntry};
use std::{
    collections::HashMap,
    marker::PhantomData,
    path::{Path, PathBuf},
};

pub struct EndpointBuilder {
    pub(crate) path: Option<PathBuf>,
    pub(crate) silent: bool,
    pub(crate) disabled: bool,
}

impl EndpointBuilder {
    pub fn no_path(mut self) -> Self {
        self.path = None;
        self
    }

    pub fn path(mut self, path: impl AsRef<Path>) -> Self {
        self.path = Some(path.as_ref().into());
        self
    }

    pub fn silent(mut self) -> Self {
        self.silent = true;
        self
    }

    pub fn disable(mut self) -> Self {
        self.disabled = true;
        self
    }
}

pub struct LoggerBuilder<EP: EndpointSuper> {
    pub(crate) base_path: Option<PathBuf>,
    pub(crate) fmt: fn(LogEntry<EP>) -> String,
    pub(crate) endpoints: HashMap<EndpointHash, EndpointBuilder>,
}

impl<EP: EndpointSuper> LoggerBuilder<EP> {
    pub fn format(mut self, fmt: fn(LogEntry<EP>) -> String) -> Self {
        self.fmt = fmt;
        self
    }

    pub fn base_path(mut self, path: impl AsRef<Path>) -> Self {
        self.base_path = Some(path.as_ref().into());
        self
    }

    pub fn setup(mut self, endpoint: EP, setup: fn(EndpointBuilder) -> EndpointBuilder) -> Self {
        self.endpoints.insert(
            crate::hash(&endpoint),
            setup(EndpointBuilder {
                path: None,
                silent: false,
                disabled: false,
            }),
        );
        self
    }

    pub fn build(self) -> KeepAlive {
        crate::setup_logger(self);
        KeepAlive(PhantomData)
    }
}
