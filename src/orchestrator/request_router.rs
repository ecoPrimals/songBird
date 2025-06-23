use std::sync::Arc;
use tokio::time::{timeout, Duration};
use std::collections::HashMap;
use std::sync::atomic::AtomicU64;

use crate::load_balancer::{LoadBalancer, ServiceInstance};
use crate::traits::communication::CommunicationLayer;
use crate::traits::service::{ServiceRequest, ServiceResponse, ResponseStatus};
use crate::errors::{Result, SongbirdError};

#[derive(Clone)]
pub struct RequestRouter {
    load_balancer: Arc<dyn LoadBalancer>,
    communication: Arc<dyn CommunicationLayer>,
    config: RequestRouterConfig,
    metrics: Arc<RequestMetrics>,
}

#[derive(Debug, Clone)]
pub struct RequestRouterConfig {
    pub default_timeout: Duration,
    pub max_retries: u32,
    pub retry_delay: Duration,
    pub enable_request_tracing: bool,
}

impl Default for RequestRouterConfig {
    fn default() -> Self {
        Self {
            default_timeout: Duration::from_secs(30),
            max_retries: 3,
            retry_delay: Duration::from_millis(100),
            enable_request_tracing: true,
        }
    }
}

impl RequestRouter {
    pub fn new(
        load_balancer: Arc<dyn LoadBalancer>,
        communication: Arc<dyn CommunicationLayer>,
    ) -> Self {
        Self {
            load_balancer,
            communication,
            config: RequestRouterConfig::default(),
            metrics: Arc::new(RequestMetrics::default()),
        }
    }

    pub async fn route_request(
        &self,
        service_instances: &[ServiceInstance],
        mut request: ServiceRequest,
    ) -> Result<ServiceResponse> {
        let start_time = std::time::Instant::now();
        
        // Add tracing information
        if self.config.enable_request_tracing {
            request.headers.insert(
                "x-trace-id".to_string(),
                uuid::Uuid::new_v4().to_string(),
            );
            request.headers.insert(
                "x-orchestrator-timestamp".to_string(),
                chrono::Utc::now().to_rfc3339(),
            );
        }

        if service_instances.is_empty() {
            return Err(SongbirdError::Service {
                message: "No service instances available".to_string(),
            });
        }

        // Attempt request with retries
        let mut last_error = None;
        for attempt in 0..=self.config.max_retries {
            // Select service instance using load balancer
            match self.load_balancer.select_service(service_instances).await {
                Ok(Some(selected_instance)) => {
                    // Attempt to send request
                    match self.send_request_to_instance(&selected_instance, &request).await {
                        Ok(response) => {
                            // Record success metrics
                            let duration = start_time.elapsed();
                            self.metrics.record_success(&selected_instance.service_info.id, duration);
                            return Ok(response);
                        }
                        Err(e) => {
                            // Record failure metrics
                            self.metrics.record_failure(&selected_instance.service_info.id);
                            last_error = Some(e);
                            if attempt < self.config.max_retries {
                                tokio::time::sleep(self.config.retry_delay).await;
                            }
                        }
                    }
                }
                Ok(None) => {
                    return Err(SongbirdError::Service {
                        message: "No healthy service instances available".to_string(),
                    });
                }
                Err(e) => {
                    return Err(SongbirdError::Service {
                        message: format!("Load balancer error: {}", e),
                    });
                }
            }
        }

        // All retries failed
        Err(last_error.unwrap_or_else(|| {
            SongbirdError::Service {
                message: "All retry attempts failed".to_string(),
            }
        }))
    }

    async fn send_request_to_instance(
        &self,
        instance: &ServiceInstance,
        request: &ServiceRequest,
    ) -> Result<ServiceResponse> {
        use crate::traits::communication::{ServiceAddress, ServiceMessage, MessageType};

        let service_address = ServiceAddress {
            service_id: instance.service_info.id.clone(),
            instance_id: Some(instance.service_info.id.clone()),
            endpoint: instance.service_info.endpoints.first()
                .map(|e| e.path.clone()),
        };

        // Prepare headers with HTTP routing information
        let mut headers = request.headers.clone();
        headers.insert("x-request-path".to_string(), request.path.clone());
        headers.insert("x-request-method".to_string(), request.method.clone());
        headers.insert("x-target-service".to_string(), instance.service_info.id.clone());

        // Convert ServiceRequest to ServiceMessage for communication layer
        let mut message = ServiceMessage {
            id: request.id.clone(),
            message_type: MessageType::Request,
            topic: None,
            payload: request.payload.clone(),
            headers,
            timestamp: request.timestamp,
            correlation_id: Some(uuid::Uuid::new_v4().to_string()),
            reply_to: None,
            ttl: request.timeout.map(|d| d.as_secs()),
        };

        // For HTTP communication, embed the request details in the payload
        if let Ok(combined_payload) = serde_json::to_value(&serde_json::json!({
            "method": request.method,
            "path": request.path,
            "headers": request.headers,
            "payload": request.payload,
            "request_id": request.id,
            "timestamp": request.timestamp,
            "metadata": request.metadata
        })) {
            message.payload = combined_payload;
        }

        tracing::debug!(
            service_id = %instance.service_info.id,
            request_id = %request.id,
            method = %request.method,
            path = %request.path,
            "Routing request to service instance"
        );

        // Send request with timeout
        let timeout_duration = request.timeout.unwrap_or(self.config.default_timeout);
        let comm_response = timeout(
            timeout_duration,
            self.communication.send_message(service_address, message),
        )
        .await
        .map_err(|_| {
            tracing::warn!(
                service_id = %instance.service_info.id,
                request_id = %request.id,
                timeout_secs = timeout_duration.as_secs(),
                "Request timeout"
            );
            SongbirdError::Service {
                message: format!("Request timeout after {:?}", timeout_duration),
            }
        })?
        .map_err(|e| {
            tracing::warn!(
                service_id = %instance.service_info.id,
                request_id = %request.id,
                error = %e,
                "Communication failed"
            );
            SongbirdError::Service {
                message: format!("Communication failed: {}", e),
            }
        })?;

        tracing::debug!(
            service_id = %instance.service_info.id,
            request_id = %request.id,
            success = comm_response.success,
            "Received communication response"
        );

        // Convert CommunicationResponse back to ServiceResponse
        let response = ServiceResponse {
            request_id: request.id.clone(),
            status: if comm_response.success {
                ResponseStatus::Success
            } else {
                ResponseStatus::Error {
                    code: if let Some(payload) = &comm_response.payload {
                        payload.get("status_code")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(500) as u16
                    } else {
                        500
                    },
                    message: comm_response
                        .error
                        .unwrap_or_else(|| "Unknown error".to_string()),
                }
            },
            headers: HashMap::new(),
            payload: comm_response.payload.unwrap_or(serde_json::json!(null)),
            timestamp: comm_response.timestamp,
            duration: chrono::Utc::now()
                .signed_duration_since(request.timestamp)
                .to_std()
                .unwrap_or_default(),
            processing_time: 0, // Will be calculated by receiving service
            metadata: HashMap::new(),
        };

        Ok(response)
    }

    pub fn get_metrics(&self) -> RequestMetrics {
        (*self.metrics).clone()
    }
}

#[derive(Debug, Default)]
pub struct RequestMetrics {
    pub total_requests: std::sync::atomic::AtomicU64,
    pub successful_requests: std::sync::atomic::AtomicU64,
    pub failed_requests: std::sync::atomic::AtomicU64,
    pub average_response_time: std::sync::atomic::AtomicU64, // milliseconds
    pub requests_by_service: parking_lot::RwLock<HashMap<String, ServiceRequestMetrics>>,
    pub total_response_time: std::sync::atomic::AtomicU64, // milliseconds
}

impl Clone for RequestMetrics {
    fn clone(&self) -> Self {
        use std::sync::atomic::Ordering;
        
        Self {
            total_requests: AtomicU64::new(self.total_requests.load(Ordering::Relaxed)),
            successful_requests: AtomicU64::new(self.successful_requests.load(Ordering::Relaxed)),
            failed_requests: AtomicU64::new(self.failed_requests.load(Ordering::Relaxed)),
            average_response_time: AtomicU64::new(self.average_response_time.load(Ordering::Relaxed)),
            requests_by_service: parking_lot::RwLock::new(self.requests_by_service.read().clone()),
            total_response_time: AtomicU64::new(self.total_response_time.load(Ordering::Relaxed)),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ServiceRequestMetrics {
    pub total: u64,
    pub successful: u64,
    pub failed: u64,
    pub average_response_time: u64,
}

impl RequestMetrics {
    fn record_success(&self, service_id: &str, duration: Duration) {
        use std::sync::atomic::Ordering;
        
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        self.successful_requests.fetch_add(1, Ordering::Relaxed);
        
        let duration_ms = duration.as_millis() as u64;
        // Simple moving average approximation
        let current_avg = self.average_response_time.load(Ordering::Relaxed);
        let new_avg = if current_avg == 0 {
            duration_ms
        } else {
            (current_avg * 9 + duration_ms) / 10 // Exponential moving average
        };
        self.average_response_time.store(new_avg, Ordering::Relaxed);
        
        // Update per-service metrics
        let mut service_metrics = self.requests_by_service.write();
        let entry = service_metrics.entry(service_id.to_string())
            .or_insert_with(ServiceRequestMetrics::default);
        entry.total += 1;
        entry.successful += 1;
        entry.average_response_time = 
            (entry.average_response_time * (entry.total - 1) + duration_ms) / entry.total;
    }

    fn record_failure(&self, service_id: &str) {
        use std::sync::atomic::Ordering;
        
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        self.failed_requests.fetch_add(1, Ordering::Relaxed);
        
        // Update per-service metrics
        let mut service_metrics = self.requests_by_service.write();
        let entry = service_metrics.entry(service_id.to_string())
            .or_insert_with(ServiceRequestMetrics::default);
        entry.total += 1;
        entry.failed += 1;
    }
} 