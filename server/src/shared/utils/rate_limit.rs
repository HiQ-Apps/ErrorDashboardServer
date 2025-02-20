use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use tokio::time::interval;

pub struct DynamicStripedRateLimiter {
    stripes: Arc<RwLock<Vec<Arc<Mutex<HashMap<String, (usize, Instant)>>>>>>,
    num_stripes: usize,

    rate_limit_duration: Duration,
    max_requests_per_ip: usize,
    cleanup_interval: Duration,
}

impl DynamicStripedRateLimiter {
    pub fn new(
        num_stripes: usize,
        rate_limit_duration: Duration,
        max_requests_per_ip: usize,
        cleanup_interval: Duration,
    ) -> Arc<Self> {
        let stripes = Self::initialize_stripes(num_stripes);

        let limiter = Arc::new(DynamicStripedRateLimiter {
            stripes: Arc::new(RwLock::new(stripes)),
            num_stripes,
            rate_limit_duration,
            max_requests_per_ip,
            cleanup_interval,
        });

        // Start the cleanup task
        Self::start_cleanup_task(Arc::clone(&limiter));

        limiter
    }

    fn initialize_stripes(
        num_stripes: usize,
    ) -> Vec<Arc<Mutex<HashMap<String, (usize, Instant)>>>> {
        let mut stripes = Vec::with_capacity(num_stripes);
        for _ in 0..num_stripes {
            stripes.push(Arc::new(Mutex::new(HashMap::new())));
        }
        stripes
    }

    fn hash_ip(&self, ip: &str) -> usize {
        let mut hasher = DefaultHasher::new();
        ip.hash(&mut hasher);
        let hash_value = hasher.finish();
        hash_value as usize % self.num_stripes
    }

    pub fn check_rate_limit(&self, ip: &str) -> bool {
        let stripe_index = self.hash_ip(ip);
        let stripes = self.stripes.read().unwrap();
        let stripe = &stripes[stripe_index];
        let mut map = stripe.lock().unwrap();

        let now = Instant::now();

        let entry = map.entry(ip.to_string()).or_insert((0, now));

        if now.duration_since(entry.1) > self.rate_limit_duration {
            entry.0 = 0;
            entry.1 = now;
        }

        if entry.0 >= self.max_requests_per_ip {
            false
        } else {
            entry.0 += 1;
            true
        }
    }

    fn start_cleanup_task(limiter: Arc<Self>) {
        tokio::spawn(async move {
            let mut interval = interval(limiter.cleanup_interval);

            loop {
                interval.tick().await;
                limiter.cleanup_expired_entries();
            }
        });
    }

    fn cleanup_expired_entries(&self) {
        let stripes = self.stripes.read().unwrap();
        let now = Instant::now();

        for stripe in stripes.iter() {
            let mut map = stripe.lock().unwrap();
            map.retain(|_, &mut (_, timestamp)| {
                now.duration_since(timestamp) <= self.rate_limit_duration
            });
        }
    }
}
