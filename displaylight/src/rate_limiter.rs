//! Rate limiting struct that sleeps to meet a desired rate.
use std::thread::sleep;
use std::time::{Duration, Instant};

/// Rate limiter struct.
pub struct Limiter {
    start: Instant,
    period: Duration,
}

impl Limiter {
    /// Create a new rate limiter, that runs at the rate specified in Hz.
    pub fn new(rate: f32) -> Limiter {
        Limiter {
            start: Instant::now(),
            period: Duration::from_secs_f32(1.0f32 / rate),
        }
    }

    /// Sleep will sleep a duration that ensures we sleep period - (time spent since leaving previous sleep).
    /// Thereby trying to meet the desired rate as best as possible.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_limiter() {
        let mut limiter = Limiter::new(10.0);
        let mut t_old = Instant::now();
        for _i in 0..10 {
            limiter.sleep();
            let t_new = Instant::now();

            // println!("Time difference: {:?}", t_new - t_old);
            let desired: f32 = 0.1;
            const WITHIN: f32 = 0.05;
            assert!((((t_new - t_old).as_secs_f32() - desired).abs()) < WITHIN);
            t_old = t_new;

            sleep(Duration::from_secs_f32(0.05)); // sleep some extra here to ensure we do 'work'.
        }
    }
}
