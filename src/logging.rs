use std::{borrow::Cow, collections::BTreeMap};

use bon::{Builder, builder};
use clap::{crate_name, crate_version};
use logfire::{ShutdownHandler, config::SendToLogfire};
use sentry::{ClientInitGuard, ClientOptions, IntoDsn, Level, SessionMode};
use serde_json::Value;
use tracing::level_filters::LevelFilter;

use crate::prelude::*;

#[must_use]
pub struct Logging {
    #[expect(dead_code)]
    sentry_guard: ClientInitGuard,

    logfire_handler: ShutdownHandler,
}

impl Logging {
    pub fn init(sentry_dsn: Option<&str>) -> Result<Self> {
        let sentry_guard = sentry::init(ClientOptions {
            dsn: sentry_dsn.into_dsn()?,
            attach_stacktrace: true,
            trim_backtraces: false,
            in_app_include: vec![crate_name!()],
            release: Some(Cow::Borrowed(crate_version!())),
            send_default_pii: true,
            session_mode: SessionMode::Application,
            traces_sample_rate: 1.0,
            ..Default::default()
        });

        let logfire_handler = logfire::configure()
            .install_panic_handler()
            .with_default_level_filter(LevelFilter::INFO)
            .send_to_logfire(SendToLogfire::IfTokenPresent)
            .finish()?;

        if !sentry_guard.is_enabled() {
            warn!("⚠️ Sentry is not configured");
        }

        Ok(Self { sentry_guard, logfire_handler })
    }

    pub fn try_shutdown(self) -> Result {
        self.logfire_handler.shutdown()?;
        Ok(())
    }
}

#[must_use]
#[derive(Builder)]
pub struct Breadcrumb {
    /// The severity of an event.
    ///
    /// The level is set to one of five values: `fatal`, `error`, `warning`, `info`, and `debug`, in order of severity.
    #[builder(start_fn)]
    level: Level,

    /// A key-value mapping of a breadcrumb's arbitrary data.
    ///
    /// This is useful for attaching structured information related to the breadcrumb, like IDs or variable values.
    #[builder(field = BTreeMap::new())]
    data: BTreeMap<String, Value>,

    /// The event category.
    ///
    /// This data is similar to a logger name and helps you understand where an event took place, such as `auth`.
    #[builder(into)]
    category: Option<String>,

    /// A string describing the event, rendered as text with all whitespace preserved.
    ///
    /// It's often used as a drop-in for a traditional log message.
    #[builder(into)]
    message: Option<String>,
}

impl<S: breadcrumb_builder::State> BreadcrumbBuilder<S> {
    pub fn data(mut self, key: impl Into<String>, value: impl Into<Value>) -> Self {
        self.data.insert(key.into(), value.into());
        self
    }
}

impl Breadcrumb {
    pub fn debug() -> BreadcrumbBuilder {
        Self::builder(Level::Debug)
    }

    pub fn add(self) {
        sentry::add_breadcrumb(sentry::Breadcrumb {
            level: self.level,
            category: self.category,
            message: self.message,
            data: self.data,
            ..Default::default()
        });
    }
}
