use std::{borrow::Cow, io::stderr};

use clap::{crate_name, crate_version};
use logfire::ShutdownHandler;
use sentry::{ClientInitGuard, ClientOptions, SessionMode};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{EnvFilter, Layer, layer::SubscriberExt, util::SubscriberInitExt};

use crate::prelude::*;

pub fn init(sentry_dsn: Option<&str>, logfire_token: Option<String>) -> Result<LoggingGuards> {
    let sentry_options = ClientOptions {
        attach_stacktrace: true,
        in_app_include: vec![crate_name!()],
        release: Some(Cow::Borrowed(crate_version!())),
        send_default_pii: true,
        session_mode: SessionMode::Application,
        traces_sample_rate: 1.0,
        ..Default::default()
    };
    let sentry_guard = sentry::init((sentry_dsn, sentry_options));

    let stderr_guard = {
        let (stderr, stderr_guard) = tracing_appender::non_blocking(stderr());
        let subscriber_layer = {
            let format_filter =
                EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
            tracing_subscriber::fmt::layer().with_writer(stderr).with_filter(format_filter)
        };
        tracing_subscriber::Registry::default().with(subscriber_layer).try_init()?;
        stderr_guard
    };

    let shutdown_handler = if let Some(token) = logfire_token {
        Some(
            logfire::configure()
                .install_panic_handler()
                .with_token(token)
                .with_console(None)
                .finish()?,
        )
    } else {
        warn!("⚠️ Logfire is not configured");
        None
    };

    if !sentry_guard.is_enabled() {
        warn!("⚠️ Sentry is not configured");
    }

    Ok(LoggingGuards { sentry: sentry_guard, stderr: stderr_guard, logfire: shutdown_handler })
}

#[must_use]
pub struct LoggingGuards {
    #[expect(dead_code)]
    sentry: ClientInitGuard,

    #[expect(dead_code)]
    stderr: WorkerGuard,

    logfire: Option<ShutdownHandler>,
}

impl LoggingGuards {
    pub fn try_shutdown(self) -> Result {
        if let Some(logfire) = self.logfire {
            logfire.shutdown()?;
        }
        Ok(())
    }
}
