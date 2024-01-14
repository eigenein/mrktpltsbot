use std::time::Duration;

use tokio::time::{sleep_until, Instant};

pub struct Throttler {
    next_instant: Instant,
    delay: Duration,
}

impl Throttler {
    pub fn new(delay: Duration) -> Self {
        Self { next_instant: Instant::now(), delay }
    }

    pub async fn throttle(&mut self) {
        sleep_until(self.next_instant).await;
        self.next_instant = Instant::now() + self.delay;
    }

    pub fn set_delay(&mut self, delay: Duration) {
        self.delay = delay;
    }
}
