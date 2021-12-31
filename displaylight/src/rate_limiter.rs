use std::thread::sleep;
use std::time::{Duration, Instant};

pub struct Limiter {
    start: Instant,
    period: Duration,
}

impl Limiter {
    pub fn new(rate: f32) -> Limiter {
        Limiter {
            start: Instant::now(),
            period: Duration::from_secs_f32(1.0f32 / rate),
        }
    }

    pub fn sleep(&mut self) {
        let now = Instant::now();
        let since_last = now - self.start;

        if since_last < self.period {
            // actually have to rate limit, for period - what we already spent doing things.
            let delta = self.period - since_last;
            sleep(delta);
        }
        self.start = Instant::now();
    }
}
