use std::{sync::Arc, time::Duration};

use chrono::Local;
use clap::builder::TypedValueParser;
use futures::{stream, StreamExt, TryStreamExt};
use pid::Pid;
use tokio::{spawn, sync::Mutex};
use tracing::instrument;

use crate::{
    marktplaats::{models::Listing, Marktplaats},
    prelude::*,
    throttler::Throttler,
    tracing::traced_result,
};

pub struct Crawler {
    marktplaats: Marktplaats,
    throttler: Arc<Mutex<Throttler>>,
    rps_pid: Arc<Mutex<Pid<f64>>>,
}

impl Crawler {
    pub fn new(throttle_delay: Duration) -> Result<Self> {
        Ok(Self::with(Marktplaats::new()?, Throttler::new(throttle_delay)))
    }

    pub fn with(marktplaats: Marktplaats, throttler: Throttler) -> Self {
        let mut rps_pid = Pid::new(90.0, 10.0);
        rps_pid.p(-0.01, 10.0).i(-0.015, 10.0).d(-0.001, 10.0);
        Self {
            marktplaats,
            throttler: Arc::new(Mutex::new(throttler)),
            rps_pid: Arc::new(Mutex::new(rps_pid)),
        }
    }

    /// Run the crawler indefinitely.
    pub async fn run(&self) -> Result {
        let stream = stream::iter(2_069_494_300..)
            .map(|item_id| spawn(Self::crawl_item(self.marktplaats.clone(), item_id)))
            .buffer_unordered(4)
            .try_filter_map(|result| async move { Ok(traced_result(result).ok().flatten()) });

        let mut stream = Box::pin(stream);
        while let Some(listing) = stream.try_next().await? {
            let lag = Local::now() - listing.timestamp;
            let lag_secs = lag.num_milliseconds() as f64 / 1000.0; // FIXME: may be negative.
            let new_rps = self.rps_pid.lock().await.next_control_output(lag_secs).output.max(1.0);
            info!(%listing.item_id, lag_secs, new_rps, "Search yielded an item");
            self.throttle(new_rps).await;
        }

        Ok(())
    }

    async fn throttle(&self, new_rps: f64) {
        let mut throttler = self.throttler.lock().await;
        throttler.set_delay(Duration::from_secs_f64(1.0 / new_rps));
        throttler.throttle().await;
        drop(throttler);
    }

    /// Crawl a single item on Marktplaats.
    #[instrument(skip_all, fields(item_id = item_id))]
    async fn crawl_item(marktplaats: Marktplaats, item_id: u32) -> Result<Option<Listing>> {
        marktplaats
            .find_one(&format!("m{item_id}"))
            .await
            .with_context(|| format!("failed to fetch item #{item_id}"))
    }
}
