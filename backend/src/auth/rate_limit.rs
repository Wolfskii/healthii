use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const WINDOW: Duration = Duration::from_secs(15 * 60);
const MAX_ATTEMPTS: u32 = 10;

#[derive(Clone, Default)]
pub struct LoginLimiter {
    inner: Arc<Mutex<HashMap<String, Window>>>,
}

struct Window {
    started_at: Instant,
    count: u32,
}

impl LoginLimiter {
    pub fn check(&self, key: &str) -> bool {
        let mut map = match self.inner.lock() {
            Ok(guard) => guard,
            Err(_) => return true,
        };

        let now = Instant::now();
        map.retain(|_, window| now.duration_since(window.started_at) < WINDOW);

        let window = map.entry(key.to_string()).or_insert(Window {
            started_at: now,
            count: 0,
        });

        if now.duration_since(window.started_at) >= WINDOW {
            window.started_at = now;
            window.count = 0;
        }

        if window.count >= MAX_ATTEMPTS {
            return false;
        }

        window.count += 1;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_after_max_attempts() {
        let limiter = LoginLimiter::default();
        for _ in 0..MAX_ATTEMPTS {
            assert!(limiter.check("10.0.0.1"));
        }
        assert!(!limiter.check("10.0.0.1"));
        assert!(limiter.check("10.0.0.2"));
    }
}
