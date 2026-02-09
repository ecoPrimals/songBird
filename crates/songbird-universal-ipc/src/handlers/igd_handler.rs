//! IGD (Internet Gateway Device) JSON-RPC handler
//!
//! Provides JSON-RPC methods for automatic router port forwarding
//! via UPnP IGD (RFC 6970) and NAT-PMP (RFC 6886).
//!
//! **Methods**:
//! - `igd.discover` - Discover router IGD capabilities
//! - `igd.map_port` - Request port forwarding
//! - `igd.unmap_port` - Remove port forwarding
//! - `igd.status` - Query all current mappings
//! - `igd.external_ip` - Get external IP from router
//! - `igd.auto_configure` - All-in-one setup + verify

use serde_json::{json, Value};
use songbird_igd::{Gateway, GatewayProtocol, PortMapping};
use songbird_igd::renewal::RenewalManager;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// IGD handler for JSON-RPC integration
///
/// Manages router port forwarding discovery and configuration.
/// Follows the same pattern as `StunHandler`.
#[derive(Debug)]
pub struct IgdHandler {
    /// Currently discovered gateway
    gateway: Arc<RwLock<Option<Gateway>>>,
    /// Active port mappings
    renewal_manager: Arc<RenewalManager>,
}

impl IgdHandler {
    /// Create new IGD handler
    pub fn new() -> Self {
        Self {
            gateway: Arc::new(RwLock::new(None)),
            renewal_manager: Arc::new(RenewalManager::new()),
        }
    }

    /// Handle `igd.discover` - Discover router IGD capabilities
    pub async fn handle_discover(&self, _params: Value) -> Value {
        info!("IGD: Discovering router capabilities");

        let (gateway, diagnostics) = Gateway::discover_with_diagnostics().await;

        let protocol_name = match &gateway.protocol {
            GatewayProtocol::UpnpIgd { .. } => "upnp_igd",
            GatewayProtocol::NatPmp => "nat_pmp",
            GatewayProtocol::None => "none",
        };

        // Store the gateway for subsequent operations
        *self.gateway.write().await = Some(gateway.clone());

        match &gateway.protocol {
            GatewayProtocol::UpnpIgd { control_url, device_name, .. } => {
                json!({
                    "protocol": protocol_name,
                    "gateway_ip": gateway.ip.to_string(),
                    "control_url": control_url,
                    "external_ip": gateway.external_ip.map(|ip| ip.to_string()),
                    "device_friendly_name": device_name,
                    "capabilities": ["AddPortMapping", "DeletePortMapping", "GetExternalIPAddress"]
                })
            }
            GatewayProtocol::NatPmp => {
                json!({
                    "protocol": protocol_name,
                    "gateway_ip": gateway.ip.to_string(),
                    "external_ip": gateway.external_ip.map(|ip| ip.to_string()),
                    "capabilities": ["MapPort", "GetExternalIP"]
                })
            }
            GatewayProtocol::None => {
                json!({
                    "protocol": "none",
                    "gateway_ip": diagnostics.gateway_ip.to_string(),
                    "upnp_tried": diagnostics.upnp_ssdp_sent,
                    "upnp_devices_found": diagnostics.upnp_devices_found,
                    "upnp_igd_found": false,
                    "nat_pmp_tried": diagnostics.nat_pmp_sent,
                    "nat_pmp_responded": diagnostics.nat_pmp_responded,
                    "recommendation": format!(
                        "Enable UPnP on your router, or manually forward TCP port 3492 to your local IP"
                    ),
                    "manual_config": {
                        "router_admin": format!("http://{}", diagnostics.gateway_ip),
                        "steps": diagnostics.manual_instructions
                    },
                    "alternative_tiers": diagnostics.alternative_tiers
                })
            }
        }
    }

    /// Handle `igd.map_port` - Request port forwarding
    pub async fn handle_map_port(&self, params: Value) -> Value {
        let external_port = params["external_port"].as_u64().unwrap_or(3492) as u16;
        let internal_port = params["internal_port"].as_u64().unwrap_or(external_port as u64) as u16;
        let protocol = params["protocol"].as_str().unwrap_or("TCP");
        let description = params["description"]
            .as_str()
            .unwrap_or("Songbird sovereign beacon");
        let ttl = params["ttl"].as_u64().unwrap_or(86400) as u32;

        info!(
            "IGD: Mapping port {} {} -> :{}",
            protocol, external_port, internal_port
        );

        let gateway = self.gateway.read().await;
        let gateway = match gateway.as_ref() {
            Some(gw) => gw,
            None => {
                // Auto-discover if not yet discovered
                drop(gateway);
                self.handle_discover(Value::Null).await;
                let gateway = self.gateway.read().await;
                match gateway.as_ref() {
                    Some(gw) => {
                        if !gw.is_available() {
                            return json!({
                                "error": "No IGD-capable gateway found",
                                "suggestion": "Enable UPnP on your router or forward port manually"
                            });
                        }
                        // Need to drop and re-acquire to avoid borrow issues
                        drop(gateway);
                        let gw = self.gateway.read().await;
                        let gw = gw.as_ref().unwrap();
                        return self.do_map_port(gw, external_port, internal_port, protocol, ttl).await;
                    }
                    None => {
                        return json!({"error": "Gateway discovery failed"});
                    }
                }
            }
        };

        self.do_map_port(gateway, external_port, internal_port, protocol, ttl).await
    }

    async fn do_map_port(
        &self,
        gateway: &Gateway,
        external_port: u16,
        internal_port: u16,
        protocol: &str,
        ttl: u32,
    ) -> Value {
        match gateway.map_port(external_port, internal_port, protocol, ttl).await {
            Ok(mapping) => {
                // Add to renewal manager
                self.renewal_manager.add_mapping(mapping.clone()).await;

                json!({
                    "mapped": true,
                    "protocol_used": match &gateway.protocol {
                        GatewayProtocol::UpnpIgd { .. } => "upnp_igd",
                        GatewayProtocol::NatPmp => "nat_pmp",
                        GatewayProtocol::None => "none",
                    },
                    "external": format!(
                        "{}:{}",
                        mapping.external_ip.map(|ip| ip.to_string()).unwrap_or_default(),
                        mapping.external_port
                    ),
                    "internal": format!("{}:{}", mapping.internal_client, mapping.internal_port),
                    "ttl": mapping.lease_duration,
                    "description": mapping.description
                })
            }
            Err(e) => {
                json!({
                    "mapped": false,
                    "error": e.to_string()
                })
            }
        }
    }

    /// Handle `igd.unmap_port` - Remove port forwarding
    pub async fn handle_unmap_port(&self, params: Value) -> Value {
        let external_port = params["external_port"].as_u64().unwrap_or(3492) as u16;
        let protocol = params["protocol"].as_str().unwrap_or("TCP");

        info!("IGD: Unmapping port {} {}", protocol, external_port);

        let gateway = self.gateway.read().await;
        match gateway.as_ref() {
            Some(gw) => match gw.unmap_port(external_port, protocol).await {
                Ok(()) => {
                    self.renewal_manager.remove_mapping(external_port).await;
                    json!({"unmapped": true})
                }
                Err(e) => json!({"unmapped": false, "error": e.to_string()}),
            },
            None => json!({"unmapped": false, "error": "No gateway discovered"}),
        }
    }

    /// Handle `igd.status` - Query all mappings and gateway state
    pub async fn handle_status(&self, _params: Value) -> Value {
        let gateway = self.gateway.read().await;
        let mappings = self.renewal_manager.get_mappings().await;

        let mappings_json: Vec<Value> = mappings
            .iter()
            .map(|m| {
                json!({
                    "external_port": m.external_port,
                    "internal_port": m.internal_port,
                    "internal_ip": m.internal_client.to_string(),
                    "protocol": m.protocol.as_str(),
                    "description": m.description,
                    "ttl_remaining": m.time_until_expiration().as_secs(),
                    "active": m.active
                })
            })
            .collect();

        match gateway.as_ref() {
            Some(gw) => json!({
                "gateway_ip": gw.ip.to_string(),
                "external_ip": gw.external_ip.map(|ip| ip.to_string()),
                "protocol": match &gw.protocol {
                    GatewayProtocol::UpnpIgd { .. } => "upnp_igd",
                    GatewayProtocol::NatPmp => "nat_pmp",
                    GatewayProtocol::None => "none",
                },
                "mappings": mappings_json,
                "mapping_count": mappings.len()
            }),
            None => json!({
                "gateway_ip": null,
                "protocol": "not_discovered",
                "mappings": [],
                "mapping_count": 0,
                "note": "Call igd.discover first"
            }),
        }
    }

    /// Handle `igd.external_ip` - Quick external IP query from router
    pub async fn handle_external_ip(&self, _params: Value) -> Value {
        let gateway = self.gateway.read().await;
        match gateway.as_ref() {
            Some(gw) => match gw.get_external_ip().await {
                Ok(ip) => json!({
                    "external_ip": ip.to_string(),
                    "source": match &gw.protocol {
                        GatewayProtocol::UpnpIgd { .. } => "upnp_igd",
                        GatewayProtocol::NatPmp => "nat_pmp",
                        GatewayProtocol::None => "none",
                    }
                }),
                Err(e) => json!({"error": e.to_string()}),
            },
            None => json!({"error": "No gateway discovered. Call igd.discover first"}),
        }
    }

    /// Handle `igd.auto_configure` - All-in-one setup + verify
    pub async fn handle_auto_configure(&self, params: Value) -> Value {
        let port = params["port"].as_u64().unwrap_or(3492) as u16;
        let protocol = params["protocol"].as_str().unwrap_or("TCP");

        info!("IGD: Auto-configuring port {} {}", protocol, port);

        // Step 1: Discover
        let discover_result = self.handle_discover(Value::Null).await;

        let gateway = self.gateway.read().await;
        let gw = match gateway.as_ref() {
            Some(gw) => gw,
            None => {
                return json!({
                    "configured": false,
                    "reason": "discovery_failed",
                    "recommendation": "Check network connectivity"
                });
            }
        };

        if !gw.is_available() {
            return json!({
                "configured": false,
                "reason": "no_igd_support",
                "gateway": gw.ip.to_string(),
                "discovery_details": discover_result,
                "recommendation": "Enable UPnP on router, or manually forward TCP port to your local IP",
                "fallback_tiers": [
                    "Sovereign onion: .onion address via onion.start (works everywhere, no port forward needed)",
                    "STUN hole-punch: punch.request (works for non-symmetric NAT)",
                    "Family relay: mesh via other connected family device"
                ]
            });
        }

        // Step 2: Map port
        let map_result = self.do_map_port(gw, port, port, protocol, 86400).await;

        if map_result["mapped"].as_bool() != Some(true) {
            return json!({
                "configured": false,
                "reason": "mapping_failed",
                "gateway": gw.ip.to_string(),
                "protocol_used": match &gw.protocol {
                    GatewayProtocol::UpnpIgd { .. } => "upnp_igd",
                    GatewayProtocol::NatPmp => "nat_pmp",
                    GatewayProtocol::None => "none",
                },
                "error": map_result["error"],
                "recommendation": "Check if another device has the port mapped, or try a different port"
            });
        }

        // Step 3: Return success
        json!({
            "configured": true,
            "gateway": gw.ip.to_string(),
            "protocol_used": match &gw.protocol {
                GatewayProtocol::UpnpIgd { .. } => "upnp_igd",
                GatewayProtocol::NatPmp => "nat_pmp",
                GatewayProtocol::None => "none",
            },
            "external_endpoint": map_result["external"],
            "auto_renew_enabled": true,
            "mapping": map_result
        })
    }
}

impl Default for IgdHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_igd_handler_creation() {
        let handler = IgdHandler::new();
        
        // Status should show no gateway initially
        let status = handler.handle_status(Value::Null).await;
        assert_eq!(status["protocol"], "not_discovered");
        assert_eq!(status["mapping_count"], 0);
    }
}

