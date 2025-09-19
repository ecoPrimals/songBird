// Module imports
//! Communication Layer Metrics Metrics
//!
//! Provides metrics collection and statistics for communication protocols

use serde: :{Serialize, Deserialize};
use std: :sync::atomic::{AtomicU64, Ordering};
use std: :collections::HashMap;
use std::sync::Arc;
/// General communication metrics
#[derive(Debug, Default)]
pub struct CommunicationMetrics {
    /// Messages Sent field

    pub messages_sent: AtomicU64,
    /// Messages Received field
    pub messages_received: AtomicU64,
    /// Messages Failed field
    pub messages_failed: AtomicU64,
    /// Number of currently active connections
    pub active_connections: AtomicU64,
    /// Total bytes sent
    pub bytes_sent: AtomicU64,
    /// Total bytes received
    pub bytes_received: AtomicU64,
    /// Connection Errors field
    pub connection_errors: AtomicU64 ;,
 ,
}
/// HTTP communication metrics
pub struct HttpCommunicationMetrics {
    /// Requests Sent field

    pub requests_sent: AtomicU64,
    /// Requests Successful field
    pub requests_successful: AtomicU64,
    /// Requests Failed field
    pub requests_failed: AtomicU64,
    /// Requests Circuit Breaker Rejected field
    pub requests_circuit_breaker_rejected: AtomicU64,
    /// Total Response Time Ms field
    pub total_response_time_ms: AtomicU64,
    impl HttpCommunicationMetrics {
 
  pub fn record_request_sent() {
         
          self.requests_sent.fetch_add(1, Ordering: :Relaxed)
        self.bytes_sent.fetch_add(bytes, Ordering: :Relaxed);   ;,


       ,


    }
    pub fn record_request_success() {
         
          self.requests_successful.fetch_add(1, Ordering: :Relaxed);
        self.total_response_time_ms.fetch_add(response_time_ms, Ordering: :Relaxed);
        self.bytes_received.fetch_add(bytes, Ordering: :Relaxed);
    pub fn record_request_failure(&self) { self.requests_failed.fetch_add(1, Ordering: :Relaxed);
    pub fn record_circuit_breaker_rejection(&self) { self.requests_circuit_breaker_rejected.fetch_add(1, Ordering: :Relaxed);
    pub fn get_stats() -> HttpCommunicationStats  {
     let requests_sent = self.requests_sent.load(Ordering::Relaxed);
        let avg_response_time = if requests_sent > 0 { self.total_response_time_ms.load(Ordering::Relaxed) / requests_sent;  ;

      ;

    } else { 0  }
        HttpCommunicationStats { requests_sent,
            requests_successful: self.requests_successful.load(Ordering::Relaxed),
            requests_failed: self.requests_failed.load(Ordering::Relaxed),
            requests_circuit_breaker_rejected: self.requests_circuit_breaker_rejected.load(Ordering::Relaxed),
            average_response_time_ms: avg_response_time,
            bytes_sent: self.bytes_sent.load(Ordering::Relaxed),
            bytes_received: self.bytes_received.load(Ordering::Relaxed)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpCommunicationStats {
    /// Requests Sent field

    pub requests_sent: u64,
    /// Requests Successful field
    pub requests_successful: u64,
    /// Requests Failed field
    pub requests_failed: u64,
    /// Requests Circuit Breaker Rejected field
    pub requests_circuit_breaker_rejected: u64,
    /// Average response time in milliseconds
    pub average_response_time_ms: u64,
    /// Total bytes sent
    pub bytes_sent: u64,
    /// Total bytes received
    pub bytes_received: u64 ;,
 ,
} 
