use std::time::Instant;

pub struct RateLimiter {
    last_tick: Instant,
    target_rps: f64,
}

impl RateLimiter {}
