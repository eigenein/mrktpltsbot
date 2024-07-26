use std::{borrow::Cow, io::stderr};

use clap::{crate_name, crate_version};
use sentry::{integrations::tracing::EventFilter, ClientInitGuard, ClientOptions, SessionMode};
use tracing::{Level, Metadata};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

use crate::prelude::*;

pub fn init(sentry_dsn: Option<&str>) -> Result<(ClientInitGuard, WorkerGuard)> {
    let sentry_options = ClientOptions {
        attach_stacktrace: true,
        in_app_include: vec![crate_name!()],
        release: Some(Cow::Borrowed(crate_version!())),
        send_default_pii: true,
        session_mode: SessionMode::Application,
        ..Default::default()
    };
    let sentry_guard = sentry::init((sentry_dsn, sentry_options));
    let sentry_layer = sentry::integrations::tracing::layer()
        .event_filter(event_filter)
        .span_filter(|metadata| metadata.level() >= &Level::DEBUG);

    let format_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let (stderr, stderr_guard) = tracing_appender::non_blocking(stderr());
    let subscriber_layer = tracing_subscriber::fmt::layer()
        .with_writer(stderr)
        .without_time()
        .with_filter(format_filter);

    tracing_subscriber::Registry::default()
        .with(sentry_layer)
        .with(subscriber_layer)
        .try_init()?;

    if !sentry_guard.is_enabled() {
        warn!("⚠️ Sentry is disabled");
    }
    Ok((sentry_guard, stderr_guard))
}

#[must_use]
fn event_filter(metadata: &Metadata) -> EventFilter {
    match *metadata.level() {
        Level::ERROR | Level::WARN => EventFilter::Exception,
        _ => EventFilter::Breadcrumb,
    }
}
