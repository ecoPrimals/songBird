/// Production LAN Gaming Security Management
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration, Instant};

#[derive(Debug)]
pub struct SecurityManager {
    pub rate_limiters: Arc<RwLock<HashMap<SocketAddr, RateLimiter>>>,
    pub session_keys: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}

#[derive(Debug)]
pub struct RateLimiter {
    pub requests: Vec<Instant>,
    pub max_requests: u32,
    pub window_duration: Duration,
}

impl Default for SecurityManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SecurityManager {
    pub fn new() -> Self {
        Self {
            rate_limiters: Arc::new(RwLock::new(HashMap::new())),
            session_keys: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn check_rate_limit(&self, addr: SocketAddr, max_requests: u32) -> bool {
        let mut limiters = self.rate_limiters.write().await;
        let limiter = limiters.entry(addr).or_insert_with(|| RateLimiter {
            requests: Vec::new(),
            max_requests,
            window_duration: Duration::from_secs(60),
        });

        let now = Instant::now();
        limiter
            .requests
            .retain(|&time| now.duration_since(time) < limiter.window_duration);

        if limiter.requests.len() < max_requests as usize {
            limiter.requests.push(now);
            true
        } else {
            false
        }
    }

    pub async fn generate_session_key(&self, session_id: &str) -> Vec<u8> {
        use rand::Rng;
        let key: Vec<u8> = (0..32).map(|_| rand::thread_rng().gen()).collect();

        let mut keys = self.session_keys.write().await;
        keys.insert(session_id.to_string(), key.clone());

        key
    }

    pub async fn get_session_key(&self, session_id: &str) -> Option<Vec<u8>> {
        let keys = self.session_keys.read().await;
        keys.get(session_id).cloned()
    }
}
