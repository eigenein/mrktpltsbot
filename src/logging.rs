use std::{borrow::Cow, collections::BTreeMap};

use bon::Builder;
use clap::{crate_name, crate_version};
use logfire::{
    ShutdownGuard,
    config::{ConsoleOptions, SendToLogfire},
};
use sentry::{ClientInitGuard, ClientOptions, IntoDsn, SessionMode};
use serde_json::Value;
use tracing::level_filters::LevelFilter;

use crate::prelude::*;

#[must_use]
pub struct Logging {
    #[expect(dead_code)]
    sentry_guard: ClientInitGuard,

    #[expect(dead_code)]
    logfire_guard: ShutdownGuard,
}

impl Logging {
    pub fn init(sentry_dsn: Option<&str>) -> Result<Self> {
        let sentry_guard = sentry::init(ClientOptions {
            dsn: sentry_dsn.into_dsn()?,
            attach_stacktrace: true,
            in_app_include: vec![crate_name!()],
            release: Some(Cow::Borrowed(crate_version!())),
            send_default_pii: true,
            auto_session_tracking: true,
            session_mode: SessionMode::Application,
            traces_sample_rate: 1.0,
            ..Default::default()
        });

        let logfire_guard = logfire::configure()
            .with_default_level_filter(LevelFilter::INFO)
            .with_console(Some(
                ConsoleOptions::default()
                    .with_min_log_level(Level::INFO) // doesn't seem to work
                    .with_include_timestamps(false),
            ))
            .send_to_logfire(SendToLogfire::IfTokenPresent)
            .finish()?
            .shutdown_guard();

        if !sentry_guard.is_enabled() {
            warn!("⚠️ Sentry is not configured");
        }

        Ok(Self { sentry_guard, logfire_guard })
    }
}

#[must_use]
#[derive(Builder)]
pub struct Breadcrumb {
    /// The severity of an event.
    ///
    /// The level is set to one of five values: `fatal`, `error`, `warning`, `info`, and `debug`, in order of severity.
    #[builder(start_fn)]
    level: sentry::Level,

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
        Self::builder(sentry::Level::Debug)
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
