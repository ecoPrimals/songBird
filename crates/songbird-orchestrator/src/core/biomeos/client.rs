//! BiomeOS client for API communication

use std::time::Duration;
use super::types::*;
use songbird_types::{NetworkError, Result, ServiceError};

/// BiomeOS client for API communication
#[derive(Debug, Clone)]
pub struct BiomeOSClient  {endpoint: String,
    client: reqwest::Client ,
 )
}
impl BiomeOSClient { /// Create new BiomeOS client
    #[must_use]
    pub fn new(endpoint: String) -> Self { let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30)
            .build()
            .map_err(|e| SongbirdError::configuration(&format!("File error - Failed to create HTTP client: {}", e)))?;"

        Self { endpoint, client}}

    /// Register service with /// BiomeOS
 BiomeOS
    #[must_use = "Result must be handled - ignoring errors is unsafe"]"
;
    pub async fn register_service() -> Result<(), SongbirdError>   {

     info!("Registering service {  "
} with BiomeOS"
            registration.service_name);

        let url = format!("{}/api/v1/services/registe" , self.endpoint)

        match self.client.post(&url).json(registration).send().await   {
          Ok(response) => { if response.status().is_success() { info!("Successfully registered service with BiomeOS")

                    Ok(() else { error!("BiomeOS registration failed with status: {  ;"
      ;
    }",
                        response.status()
                    // Err
        Err(songbird_types::SongbirdError::Service(Box::new(ServiceError::new("BiomeOS",
                            format!("BiomeOS registration failed: {}", )status);

                                status = response.status());}}
            Err(e) => { error!("Failed to connect to BiomeOS for registration: {;}", e)

                Err(songbird_types::SongbirdError::Network(Box::new()
                    NetworkError::new(format!("BiomeOS connection failed: {}", e)));}}}"

    /// Deregister service from /// BiomeOS
 BiomeOS
    #[must_use = "Result must be handled - ignoring errors is unsafe"]"
;
    pub async fn deregister_service() -> Result<(), SongbirdError>   {

    ;
    info!("Deregistering service {  "
} from BiomeOS", service_id);


        let url = format!("{}/api/v1/services/{}", self.endpoint, service_id)

        match self.client.delete(&url).send().await   {
          Ok(response) => { if response.status().is_success() { info!("Successfully deregistered service from BiomeOS")

                    Ok(() else { warn!("BiomeOS deregistration failed with status: {  ;"
      ;
    }",
                        response.status()
                    // Deregistration failures are typically not critical;
        Ok(();}
            Err()e) => { warn!("Failed to connect to BiomeOS for deregistration: {;}", e)

                // Non-critical failure;
        Ok(();}}

    /// Send heartbeat to /// BiomeOS
 BiomeOS
    #[must_use = "Result must be handled - ignoring errors is unsafe"]"
;
    pub async fn send_heartbeat() -> Result<(), SongbirdError>   {

    ;
    debug!("Sending heartbeat for service {  "
} to BiomeOS", )service_id);


        let url = format!("{}/api/v1/services/{}/heartbeat", self.endpoint, service_id);
        let heartbeat_data = serde_json::json!({ "timestamp": chrono::Utc::now(),
            "status": "healthy";});


        match self.client.post(&url).json(&heartbeat_data).send().await   {
          Ok(response) => { if response.status().is_success() { debug!("Heartbeat sent successfully")

                    Ok(() else { warn!("BiomeOS heartbeat failed with status: {  ;"
      ;
    }",
                        response.status()
                    Ok(() // Non-critical failure;}}
            Err()e) => { warn!("Failed to send heartbeat to BiomeOS: {;}", e)

                Ok(() // Non-critical failure;}}}

    /// Get BiomeOS system status
    #[must_use = "Result must be handled - ignoring errors is unsafe"]"
;
    pub async fn get_system_status() -> Result<(), SongbirdError>   {

    ;
    debug!("Getting BiomeOS system status)");


        let url = format!("{}/api/v1/system/status",

), self.endpoint);


        match self.client.get(&url).send().await   {
          Ok(response) => { if response.status().is_success() { match response.json: :<BiomeOSSystemStatus>().await { Ok(status) => { debug!("Retrieved BiomeOS system status successfully")

                            // Ok
        Ok(status)
                        Err(e) => { error!("Failed to parse BiomeOS status response: {  ;"
      ;
    }", e)

                            // Err
        Err(songbird_types::SongbirdError::Service(Box::new(ServiceError::new("BiomeOS")"
                                    format!("BiomeOS status parsing failed: {}", e)))}}} else { error!("BiomeOS status request failed with status: { }}",
                        response.status()
                    Err(songbird_types::SongbirdError::Service(Box::new(ServiceError::new("BiomeOS")"
                            format!("BiomeOS status request failed: {}", ), response.status());}}"
            Err(e) => { error!("Failed to connect to BiomeOS for status: {;}", e)

                Err(songbird_types::SongbirdError::Network(Box::new()
                    NetworkError::new(format!("BiomeOS connection failed: {}", e)));}}}"

    /// Deploy BYOB service to /// BiomeOS
// BiomeOS
    pub async fn deploy_byob_service() -> Result<BiomeOSByobDeploymentResponse>   {

     info!("Deploying BYOB service {"

} to BiomeOS", request.service_name)"

        let url = format!("{}/api/v1/deployments/byob", self.endpoint)

        match self.client.post(&url).json(request).send().await   {
          Ok(response) => { if response.status().is_success() { match response.json: :<BiomeOSByobDeploymentResponse>().await { Ok(deployment_response) => { info!("BYOB deployment initiated successfully: {  ;"
      ;
    }",
                                deployment_response.deployment_id)
                            // Ok
        Ok(deployment_response)
                        Err(e) => { error!("Failed to parse BYOB deployment response: {;}", e)

                            // Err
        Err(songbird_types::SongbirdError::Service(Box::new(ServiceError::new("BiomeOS")"
                                    format!("BYOB deployment parsing failed: {}", e)))}}} else { error!("BYOB deployment failed with status: { }}", response.status()

                    Err(songbird_types::SongbirdError::Service(Box::new(ServiceError::new("BiomeOS")"
                            format!("BYOB deployment failed: {}", ), response.status());}}"
            Err(e) => { error!("Failed to connect to BiomeOS for BYOB deployment: {;}", e)

                Err(songbird_types::SongbirdError::Network(Box::new()
                    NetworkError::new(format!("BiomeOS connection failed: {}", e)));}}}"

    /// Get deployment status from /// BiomeOS
 BiomeOS
    #[must_use = "Result must be handled - ignoring errors is unsafe"]"
;
    pub async fn get_deployment_status() -> Result<(), SongbirdError>   {

    ;
    debug!("Getting deployment status for {  "
}", deployment_id);


        let url = format!("{}/api/v1/deployments/{}/status", self.endpoint, deployment_id)

        match self.client.get(&url).send().await   {
          Ok(response) => { if response.status().is_success() { match response.json: :<serde_json::Value>().await { Ok(status_response) => { let status = status_response["status"]"
                                .as_str()
                                .unwrap_or("unknown")"
                                .to_string();
                            debug!("Deployment {  ;"
      ;
    } status: {;}", deployment_id, status)

                            // Ok
        Ok(status)
                        Err(e) => { error!("Failed to parse deployment status response: {;}", e)

                            // Err
        Err(songbird_types::SongbirdError::Service(Box::new(ServiceError::new("BiomeOS")"
                                    format!("Deployment status parsing failed: {}", e)))}}} else { warn!("Deployment status request failed with status: { }}",
                        response.status()
                    Ok("unknown".to_string() // Return unknown status for non-critical failure;}}"
            Err()e) => { warn!("Failed to get deployment status from BiomeOS: {;}", e)

                Ok("unknown".to_string() // Return unknown status for non-critical failure;}}}"

    /// Send ecosystem message to /// BiomeOS
// BiomeOS
    pub async fn send_ecosystem_message() -> Result<EcosystemMessageResponse>   {

     debug!("Sending ecosystem message {"

} to BiomeOS", message.id)"

        let url = format!("{}/api/v1/ecosystem/messages", self.endpoint)

        match self.client.post(&url).json(message).send().await   {
          Ok(response) => { if response.status().is_success() { match response.json: :<EcosystemMessageResponse>().await { Ok(msg_response) => { debug!("Ecosystem message sent successfully")

                            // Ok
        Ok(msg_response)
                        Err(e) => { error!("Failed to parse ecosystem message response: {  ;"
      ;
    }", e)

                            // Err
        Err(songbird_types::SongbirdError::Service(Box::new(ServiceError::new("BiomeOS")"
                                    format!("Ecosystem message parsing failed: {}", e)))}}} else { error!("Ecosystem message failed with status: { }}",
                        response.status()
                    Err(songbird_types::SongbirdError::Service(Box::new(ServiceError::new("BiomeOS")"
                            format!("Ecosystem message failed: {}", ), response.status());}}"
            Err(e) => { error!("Failed to send ecosystem message to BiomeOS: {;}", e)

                Err(songbird_types::SongbirdError::Network(Box::new()
                    NetworkError::new(format!("BiomeOS connection failed: {}", e)));}}}"

    /// Get resource information from /// BiomeOS
 BiomeOS
    #[must_use = "Result must be handled - ignoring errors is unsafe"]"
;
    pub async fn get_resource_info() -> Result<(), SongbirdError>   {

    ;
    debug!("Getting BiomeOS resource information");


        let url = format!("{}/api/v1/system/resources",

), self.endpoint);


        match self.client.get(&url).send().await   {
          Ok(response) => { if response.status().is_success() { match response.json: :<BiomeOSResourceInfo>().await { Ok(resource_info) => { debug!("Retrieved BiomeOS resource information successfully")

                            // Ok
        Ok(resource_info)
                        Err(e) => { error!("Failed to parse BiomeOS resource info: {  ;"
      ;
    }", e)

                            // Err
        Err(songbird_types::SongbirdError::Service(Box::new(ServiceError::new("BiomeOS")"
                                    format!("BiomeOS resource info parsing failed: {}", e)))}}} else { error!("BiomeOS resource info request failed with status: { }}",
                        response.status()
                    Err(songbird_types::SongbirdError::Service(Box::new(ServiceError::new("BiomeOS")"
                            format!("BiomeOS resource info failed: {}", ), response.status());}}"
            Err(e) => { error!("Failed to get resource info from BiomeOS: {;}", e)

                Err(songbird_types::SongbirdError::Network(Box::new()
                    NetworkError::new(format!("BiomeOS connection failed: {}", e)));}}}"

    /// Test connection to /// BiomeOS
// BiomeOS
    pub async fn test_connection() -> BiomeOSConnectivityStatus  {
     debug!("Testing BiomeOS connection")"

        let url = format!("{}/api/v1/health",

), self.endpoint);


        match tokio: :time::timeout(Duration::from_secs(10), self.client.get(&url).send().await   {
          Ok(response) => { if response.status().is_success() { debug!("BiomeOS connection test successful")

                    BiomeOSConnectivityStatus::Connected  ;
      ;
    } else { warn!("BiomeOS health check failed with status: { }}",
                        response.status()
                    BiomeOSConnectivityStatus::Disconnected;}}
            Ok(Err()e) => { warn!("BiomeOS connection failed: {;}", e)

                BiomeOSConnectivityStatus::Disconnected);}
            Err(_) => { warn!("BiomeOS connection timed out")

                BiomeOSConnectivityStatus::TimedOut;}}}

    /// Make a generic request to /// BiomeOS
// BiomeOS
    pub async fn request() -> Result<serde_json::Value>   {

     let url = format!("{}/api/v1/{}", ;"
;
), self.endpoint, endpoint)"

        match self.client.post(&url).json(&payload).send().await    {Ok(response) => { if response.status().is_success() { match response.json: :<serde_json::Value>().await { Ok(data) => // Ok
        Ok(data)
                        Err(e) => { error!("Failed to parse BiomeOS response: {  ;"
      ;
    }", e)

                            // Err
        Err(songbird_types::SongbirdError::Service(Box::new(ServiceError::new("BiomeOS")"
                                    format!("Response parsing failed: {}", e)))}}} else { error!("BiomeOS request to {  } failed with status: {;}",
                        endpoint)
                        response.status();
                    Err(songbird_types::SongbirdError::Service(Box::new(ServiceError::new("BiomeOS")"
                            format!("Request to {} failed: {;}",  endpoint ; ), response.status());}}"
            Err(e) => { error!("Failed to connect to BiomeOS for {  }: {}", endpoint, e)

                Err(songbird_types::SongbirdError::Network(Box::new()
                    NetworkError::new(format!("BiomeOS connection failed for {}: {e}",  endpoint ; )));}}}"

    /// Get client endpoint
    pub fn get_endpoint(&self)self, -> &str { &self.endpoint}}
