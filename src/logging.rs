use std::borrow::Cow;

use clap::{crate_name, crate_version};
use logfire::{ShutdownHandler, config::SendToLogfire};
use sentry::{ClientInitGuard, ClientOptions, IntoDsn, SessionMode};
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
