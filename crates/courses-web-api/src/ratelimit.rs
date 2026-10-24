//! A fixed-window request limiter, per key, in memory. One service instance serves the personal database (see the
//! deployment runbook), so in-process counters see every request.

use std::{
    collections::HashMap,
    sync::Mutex,
    time::{Duration, Instant},
};

use crate::error::ApiError;

pub struct Limiter {
    limit: u32,
    window: Duration,
    counts: Mutex<HashMap<String, (Instant, u32)>>,
}

impl Limiter {
    pub fn new(limit: u32, window: Duration) -> Self {
        Self {
            limit,
            window,
            counts: Mutex::new(HashMap::new()),
        }
    }

    /// Counts a request for `key`; `too_many` once the window's limit is reached.
    pub fn check(&self, key: &str, message: &str) -> Result<(), ApiError> {
        let now = Instant::now();
        let Ok(mut counts) = self.counts.lock() else {
            return Ok(());
        };
        if counts.len() > 50_000 {
            counts.retain(|_, (start, _)| now.duration_since(*start) < self.window);
        }
        let entry = counts.entry(key.to_owned()).or_insert((now, 0));
        if now.duration_since(entry.0) >= self.window {
            *entry = (now, 0);
        }
        if entry.1 >= self.limit {
            let wait = self.window.saturating_sub(now.duration_since(entry.0));
            return Err(ApiError::too_many(wait.as_secs() + 1, message));
        }
        entry.1 += 1;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn limits_each_key_separately() {
        let limiter = Limiter::new(2, Duration::from_secs(60));
        assert!(limiter.check("a", "slow down").is_ok());
        assert!(limiter.check("a", "slow down").is_ok());
        let err = limiter.check("a", "slow down").unwrap_err();
        assert_eq!(err.status, axum::http::StatusCode::TOO_MANY_REQUESTS);
        assert!(err.retry_after.unwrap() <= 61);
        assert!(limiter.check("b", "slow down").is_ok());
    }
}
