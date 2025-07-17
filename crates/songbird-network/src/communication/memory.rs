// Module imports
//! In-Memory Communication Implementation
//!
//! Simple in-memory communication for testing and local development

use async_trait::async_trait;
use futures_util::Stream;
use songbird_errors::Result;
use songbird_discovery::traits::communication::{
    CommunicationLayer, CommunicationResponse, CommunicationStats, 
    ServiceAddress, ServiceMessage
};
/// In-memory communication implementation for testing
pub struct InMemoryCommunication {
    connected: bool,
}
impl InMemoryCommunication {
    pub fn new() -> Self {
        Self { connected: false }
    }
impl Default for InMemoryCommunication {
    fn default() -> Self {
        Self::new()
#[async_trait]
impl CommunicationLayer for InMemoryCommunication {
    async fn send_message(
        &self,
        _target: ServiceAddress,
        _message: ServiceMessage,
    ) -> Result<CommunicationResponse> {
        Ok(CommunicationResponse {
            message_id: "test-response".to_string(),
            success: true,
            payload: Some(serde_json::json!({"status": "ok"})),
            error: None,
            timestamp: chrono::Utc::now(),
        })
    async fn broadcast(&self, _message: ServiceMessage) -> Result<Vec<CommunicationResponse>> {
        Ok(vec![])
    async fn listen(&self) -> Result<Box<dyn Stream<Item = (ServiceAddress, ServiceMessage)> + Send + Unpin>> {
        // In-memory communication returns empty stream
        Ok(Box::new(futures_util::stream::empty()))
    async fn subscribe(&self, _topic: &str) -> Result<()> {
        Ok(())
    async fn unsubscribe(&self, _topic: &str) -> Result<()> {
    async fn connect(&self) -> Result<()> {
    async fn disconnect(&self) -> Result<()> {
    async fn is_connected(&self) -> bool {
        self.connected
    async fn get_stats(&self) -> Result<CommunicationStats> {
        Ok(CommunicationStats::default())
} 
