use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::sync::{Arc, RwLock, Mutex};
use std::time::{Duration, Instant};
use tokio::time::interval;

 // 60 req/min
const RATE_LIMIT_DURATION: Duration = Duration::from_secs(60);
const MAX_REQUESTS_PER_IP: usize = 60;
 // Interval for cleanup task, default to half an hr
const CLEANUP_INTERVAL: Duration = Duration::from_secs(1800);

pub struct DynamicStripedRateLimiter {
    stripes: Arc<RwLock<Vec<Arc<Mutex<HashMap<String, (usize, Instant)>>>>>>,
    num_stripes: usize,
}

impl DynamicStripedRateLimiter {
    pub fn new(num_stripes: usize) -> Arc<Self> {
        let stripes = Self::initialize_stripes(num_stripes);

        let limiter = Arc::new(DynamicStripedRateLimiter {
            stripes: Arc::new(RwLock::new(stripes)),
            num_stripes,
        });

        // Start the cleanup task
        Self::start_cleanup_task(Arc::clone(&limiter));

        limiter
    }

    fn initialize_stripes(num_stripes: usize) -> Vec<Arc<Mutex<HashMap<String, (usize, Instant)>>>> {
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

        if now.duration_since(entry.1) > RATE_LIMIT_DURATION {
            entry.0 = 0;
            entry.1 = now;
        }

        if entry.0 >= MAX_REQUESTS_PER_IP {
            false
        } else {
            entry.0 += 1;
            true
        }
    }

    fn start_cleanup_task(limiter: Arc<Self>) {
        tokio::spawn(async move {
            let mut interval = interval(CLEANUP_INTERVAL);

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
            map.retain(|_, &mut (_, timestamp)| now.duration_since(timestamp) <= RATE_LIMIT_DURATION);
        }
    }
}