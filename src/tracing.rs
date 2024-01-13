use std::{borrow::Cow, io::stderr};

use clap::crate_version;
use sentry::{integrations::tracing::EventFilter, ClientInitGuard, ClientOptions, SessionMode};
use tracing::Level;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

use crate::prelude::*;

pub fn init(
    sentry_dsn: Option<String>,
    traces_sample_rate: f32,
) -> Result<(ClientInitGuard, WorkerGuard)> {
    let sentry_options = ClientOptions {
        attach_stacktrace: true,
        in_app_include: vec!["mrktpltsbot"],
        release: Some(Cow::Borrowed(crate_version!())),
        send_default_pii: true,
        session_mode: SessionMode::Application,
        traces_sample_rate,
        ..Default::default()
    };
    let sentry_guard = sentry::init((sentry_dsn, sentry_options));
    let sentry_layer = sentry::integrations::tracing::layer()
        .event_filter(|_metadata| EventFilter::Breadcrumb)
        .span_filter(|metadata| metadata.level() >= &Level::DEBUG);
    info!(is_sentry_enabled = sentry_guard.is_enabled(), "🥅");

    let format_filter = EnvFilter::try_from_default_env().unwrap_or_default();
    let (stderr, stderr_guard) = tracing_appender::non_blocking(stderr());
    let format_layer = tracing_subscriber::fmt::layer()
        .with_writer(stderr)
        .without_time()
        .with_filter(format_filter);

    tracing_subscriber::Registry::default()
        .with(sentry_layer)
        .with(format_layer)
        .try_init()?;

    Ok((sentry_guard, stderr_guard))
}
