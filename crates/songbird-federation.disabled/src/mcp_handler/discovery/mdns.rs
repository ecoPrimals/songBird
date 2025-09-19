//! mDNS/Bonjour service discovery implementation

use songbird_errors::SongbirdError;
use std::time::Duration;
use tracing::{debug, info};

use super::network_utils::send_udp_broadcast;

/// Discover federation endpoints via mDNS/Bonjour
pub async fn discover_via_mdns() -> Result<Vec<String>, SongbirdError> {
    info!("Starting mDNS/Bonjour discovery for federation endpoints");

    let mut endpoints = Vec::new();

    // mDNS service types to search for
    let service_types = vec![
        "_songbird._tcp.local",
        "_federation._tcp.local",
        "_primals._tcp.local",
        "_mcp._tcp.local",
        "_http._tcp.local", // Generic HTTP services
        "_songbird-federation._tcp.local",
    ];

    for service_type in &service_types {
        match discover_service_type(service_type).await {
            Ok(service_endpoints) => {
                let current_len = endpoints.len();
                endpoints.extend(service_endpoints);
                debug!(
                    "mDNS discovery for {} found {} endpoints",
                    service_type,
                    endpoints.len() - current_len
                );
            }
            Err(e) => {
                debug!("mDNS discovery failed for {}: {}", service_type, e);
            }
        }
    }

    // Remove duplicates
    endpoints.sort();
    endpoints.dedup();

    info!("mDNS discovery found {} unique endpoints", endpoints.len());
    Ok(endpoints)
}

/// Discover a specific mDNS service type
async fn discover_service_type(service_type: &str) -> Result<Vec<String>, SongbirdError> {
    debug!("Querying mDNS for service type: {}", service_type);

    // Create mDNS query packet
    let query = create_mdns_query(service_type).await?;

    // Send mDNS multicast query and collect responses
    let responses = send_udp_broadcast(&query, 5353, Duration::from_secs(3)).await?;

    // Parse mDNS responses to extract endpoints
    let mut endpoints = Vec::new();
    for response in responses {
        if let Ok(parsed_endpoints) = parse_mdns_response(&response).await {
            endpoints.extend(parsed_endpoints);
        }
    }

    debug!(
        "Service type {} discovered {} endpoints",
        service_type,
        endpoints.len()
    );
    Ok(endpoints)
}

/// Create mDNS query packet for a service type
async fn create_mdns_query(service_name: &str) -> Result<Vec<u8>, SongbirdError> {
    // Simplified mDNS query packet creation
    // In a real implementation, you would use a proper mDNS library
    let mut query = Vec::with_capacity(512);

    // mDNS header
    query.extend(&[0x00, 0x00]); // Transaction ID
    query.extend(&[0x01, 0x00]); // Flags (standard query)
    query.extend(&[0x00, 0x01]); // Questions count
    query.extend(&[0x00, 0x00]); // Answer RRs
    query.extend(&[0x00, 0x00]); // Authority RRs
    query.extend(&[0x00, 0x00]); // Additional RRs

    // Question section
    // Encode service name
    for part in service_name.split('.') {
        query.push(part.len() as u8);
        query.extend(part.bytes());
    }
    query.push(0); // Null terminator

    query.extend(&[0x00, 0x0C]); // Type: PTR
    query.extend(&[0x00, 0x01]); // Class: IN

    debug!("Created mDNS query packet for {}", service_name);
    Ok(query)
}

/// Parse mDNS response to extract service endpoints
async fn parse_mdns_response(response_data: &str) -> Result<Vec<String>, SongbirdError> {
    let mut endpoints = Vec::new();

    // Simple parsing - look for HTTP-like endpoints in the response
    // In a real implementation, you would properly parse mDNS packets
    let lines: Vec<&str> = response_data.lines().collect();

    for line in lines {
        if line.contains("http://") || line.contains("https://") {
            // Extract endpoint URLs
            let parts: Vec<&str> = line.split_whitespace().collect();
            for part in parts {
                if part.starts_with("http://") || part.starts_with("https://") {
                    endpoints.push(part.to_string());
                }
            }
        }

        // Look for IP:Port patterns
        if let Some(endpoint) = extract_ip_port_from_line(line) {
            endpoints.push(format!("http://{endpoint}"));
        }
    }

    debug!("Parsed {} endpoints from mDNS response", endpoints.len());
    Ok(endpoints)
}

/// Extract IP:Port patterns from text line
fn extract_ip_port_from_line(line: &str) -> Option<String> {
    // Simple regex-like parsing for IP:Port patterns
    let words: Vec<&str> = line.split_whitespace().collect();

    for word in words {
        if word.contains(':') {
            let parts: Vec<&str> = word.split(':').collect();
            if parts.len() == 2 {
                if let Ok(_port) = parts[1].parse::<u16>() {
                    // Basic IP validation
                    let ip_parts: Vec<&str> = parts[0].split('.').collect();
                    if ip_parts.len() == 4 {
                        let all_numeric = ip_parts.iter().all(|&part| part.parse::<u8>().is_ok());
                        if all_numeric {
                            return Some(word.to_string());
                        }
                    }
                }
            }
        }
    }

    None
}

/// Advertise this service via mDNS (for being discovered by others)
pub async fn advertise_via_mdns(service_port: u16) -> Result<(), SongbirdError> {
    info!(
        "Advertising Songbird federation service via mDNS on port {}",
        service_port
    );

    // Create mDNS advertisement
    let advertisement = create_mdns_advertisement(service_port).await?;

    // Send periodic mDNS advertisements
    let _responses = send_udp_broadcast(&advertisement, 5353, Duration::from_secs(1)).await?;

    debug!("mDNS advertisement sent for port {}", service_port);
    Ok(())
}

/// Create mDNS service advertisement packet
async fn create_mdns_advertisement(service_port: u16) -> Result<Vec<u8>, SongbirdError> {
    // Simplified mDNS advertisement creation
    let mut advertisement = Vec::with_capacity(512);

    // mDNS header for response
    advertisement.extend(&[0x00, 0x00]); // Transaction ID
    advertisement.extend(&[0x84, 0x00]); // Flags (response, authoritative)
    advertisement.extend(&[0x00, 0x00]); // Questions count
    advertisement.extend(&[0x00, 0x01]); // Answer RRs
    advertisement.extend(&[0x00, 0x00]); // Authority RRs
    advertisement.extend(&[0x00, 0x01]); // Additional RRs

    // Answer section - PTR record for service
    let service_name = "_songbird._tcp.local";
    for part in service_name.split('.') {
        advertisement.push(part.len() as u8);
        advertisement.extend(part.bytes());
    }
    advertisement.push(0); // Null terminator

    advertisement.extend(&[0x00, 0x0C]); // Type: PTR
    advertisement.extend(&[0x00, 0x01]); // Class: IN
    advertisement.extend(&[0x00, 0x00, 0x00, 0x78]); // TTL: 120 seconds

    // Add service instance and port information
    let instance_name = format!("songbird-{service_port}");
    advertisement.extend(&[0x00, instance_name.len() as u8]);
    advertisement.extend(instance_name.bytes());

    debug!(
        "Created mDNS advertisement for service on port {}",
        service_port
    );
    Ok(advertisement)
}
