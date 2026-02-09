//! Timing protection for Dark Forest compliance
//!
//! Prevents timing-based side-channel attacks by:
//! - Adding random delays
//! - Padding operations to constant time
//! - Using constant-time crypto (delegated to BearDog)

use crate::error::Result;
use rand::Rng;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::debug;

/// Timing protector
#[derive(Debug)]
pub struct TimingProtector {
    /// Target duration for protected operations
    target_duration: Duration,
    
    /// Maximum random delay
    max_random_delay: Duration,
    
    /// Start time (for constant-time padding)
    start_time: Option<Instant>,
}

impl TimingProtector {
    /// Create new timing protector
    pub fn new(target_duration: Duration, max_random_delay: Duration) -> Self {
        Self {
            target_duration,
            max_random_delay,
            start_time: None,
        }
    }
    
    /// Start timing protection (marks operation start)
    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
        debug!("Timing protection started");
    }
    
    /// Apply random delay (before operation)
    pub async fn random_delay(&self) {
        let delay_ms = rand::thread_rng().gen_range(0..self.max_random_delay.as_millis());
        let delay = Duration::from_millis(delay_ms as u64);
        
        debug!("Applying random delay: {:?}", delay);
        sleep(delay).await;
    }
    
    /// Pad to constant time (after operation)
    ///
    /// Sleeps remaining time to reach target duration
    pub async fn pad_to_constant_time(&self) -> Result<()> {
        if let Some(start) = self.start_time {
            let elapsed = start.elapsed();
            
            if elapsed < self.target_duration {
                let remaining = self.target_duration - elapsed;
                debug!("Padding to constant time: {:?} remaining", remaining);
                sleep(remaining).await;
            } else {
                debug!("Operation exceeded target duration: {:?} > {:?}", 
                       elapsed, self.target_duration);
            }
        }
        
        Ok(())
    }
    
    /// Full protected operation wrapper
    ///
    /// Applies random delay before + constant-time padding after
    pub async fn protect<F, T>(&mut self, f: F) -> Result<T>
    where
        F: std::future::Future<Output = Result<T>>,
    {
        // Random delay before
        self.random_delay().await;
        
        // Mark start
        self.start();
        
        // Execute operation
        let result = f.await?;
        
        // Pad to constant time
        self.pad_to_constant_time().await?;
        
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_timing_protection() {
        let mut protector = TimingProtector::new(
            Duration::from_secs(2),
            Duration::from_millis(100),
        );
        
        let start = Instant::now();
        
        protector.protect(async {
            // Fast operation
            sleep(Duration::from_millis(100)).await;
            Ok::<_, crate::error::NfcError>(())
        }).await.unwrap();
        
        let elapsed = start.elapsed();
        
        // Should be padded to at least target duration
        assert!(elapsed >= Duration::from_secs(2));
    }
}
