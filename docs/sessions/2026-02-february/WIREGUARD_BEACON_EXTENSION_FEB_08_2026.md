# WireGuard Beacon Extension

## Overview

Extended `BeaconPayload` in `songbird-discovery` to support external tunnel endpoints (WireGuard, OpenVPN, IPsec, etc.) as specified in the Multi-Path Protocol evolution.

## Implementation

### Changes to `BeaconPayload`

Added `external_tunnels` field to `crates/songbird-discovery/src/dark_forest_beacon.rs`:

```rust
pub struct BeaconPayload {
    // ... existing fields
    
    /// External tunnel endpoints (VPN/WireGuard/etc)
    ///
    /// Optional external VPN endpoints for inter-primal communication.
    /// Enables connectivity through existing VPN infrastructure.
    /// Encrypted within beacon payload for privacy.
    pub external_tunnels: Vec<ExternalTunnel>,
    
    // ... rest of fields
}
```

### New Types

```rust
/// External tunnel configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExternalTunnel {
    /// Tunnel type
    pub tunnel_type: TunnelType,
    
    /// Endpoint address (IP:port or hostname:port)
    pub endpoint: String,
    
    /// Public key (for WireGuard, base64 encoded)
    pub public_key: Option<String>,
    
    /// Optional metadata (protocol-specific)
    pub metadata: std::collections::HashMap<String, String>,
}

/// Supported external tunnel types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TunnelType {
    /// WireGuard VPN
    WireGuard,
    
    /// OpenVPN
    OpenVPN,
    
    /// IPsec
    IPsec,
    
    /// Future: ZeroTier, Tailscale, etc.
    Other(String),
}
```

## Usage

### Adding WireGuard Endpoint

```rust
use songbird_discovery::dark_forest_beacon::{BeaconPayload, ExternalTunnel, TunnelType};

// Create beacon payload with WireGuard endpoint
let payload = BeaconPayload::new(
    beacon_id,
    node_id,
    endpoints,
    capabilities,
    cluster_id,
    session_id,
)
.with_wireguard(
    "1.2.3.4:51820".to_string(),
    "base64_pubkey_here==".to_string(),
);

// Broadcast beacon - WireGuard info is encrypted!
```

### Manual External Tunnel

```rust
use std::collections::HashMap;

let mut metadata = HashMap::new();
metadata.insert("cipher".to_string(), "aes256".to_string());

let tunnel = ExternalTunnel {
    tunnel_type: TunnelType::OpenVPN,
    endpoint: "vpn.example.com:1194".to_string(),
    public_key: None,
    metadata,
};

let payload = BeaconPayload::new(/* ... */)
    .with_external_tunnel(tunnel);
```

## Dark Forest Compliance

✅ **Encrypted within beacon** - `external_tunnels` is inside `BeaconPayload`, which is encrypted before transmission  
✅ **Zero metadata leakage** - Observers cannot see tunnel type, endpoints, or public keys  
✅ **Only family members** can decrypt and use tunnel information  
✅ **Backward compatible** - Empty `external_tunnels` vec for nodes without external VPNs  

## Security Model

### Threat Model

- **Passive observers**: Cannot see tunnel endpoints (encrypted in beacon)
- **Different beacon families**: Cannot decrypt, see only noise
- **Same beacon family**: Can decrypt and discover tunnel endpoints
- **Malicious family member**: Can see tunnel endpoints (intentional - family trust model)

### Attack Surface

- **Tunnel protocol vulnerabilities**: Dependent on WireGuard/OpenVPN security
- **Endpoint enumeration**: Not possible without beacon decryption
- **Public key exposure**: Only visible to family members with beacon seed

## Multi-Path Integration

WireGuard endpoints are advertised via beacons and used in connection priority:

```
Priority Order (Sovereign Multi-Path Protocol):
1. IPv6 Direct
2. Sovereign Onion
3. IPv4 Direct
4. LAN Direct
5. STUN Hole-Punch
6. Family Relay
7. DNS Beacon Discovery
8. External Tunnels (WireGuard, etc.)  ← NEW
9. QUIC (UDP-based, faster)
```

### Connection Flow

```rust
// Multi-path connection attempt
async fn connect_to_primal(&self, address: &PrimalAddress) -> Result<Connection> {
    // ... try IPv6, Onion, IPv4, LAN, STUN ...
    
    // Try external tunnels from beacon
    if let Some(payload) = self.decrypt_beacon(address).await? {
        for tunnel in payload.external_tunnels {
            match tunnel.tunnel_type {
                TunnelType::WireGuard => {
                    if let Ok(conn) = self.connect_wireguard(&tunnel).await {
                        return Ok(conn);
                    }
                }
                _ => { /* other tunnel types */ }
            }
        }
    }
    
    // ... fallback to QUIC ...
}
```

## Future Enhancements

1. **Dynamic tunnel management**: Auto-create WireGuard interfaces on discovery
2. **Tunnel priority**: Prefer certain tunnel types based on latency/bandwidth
3. **Tunnel health checks**: Monitor tunnel availability and failover
4. **Multi-tunnel**: Use multiple tunnels simultaneously for redundancy
5. **Custom tunnels**: Support for proprietary VPN protocols via `TunnelType::Other`

## Testing

All tests pass:

```bash
$ cargo test --lib dark_forest
running 11 tests
test dark_forest_beacon::tests::test_beacon_payload_with_wireguard ... ok
test dark_forest_beacon::tests::test_beacon_payload_creation ... ok
test dark_forest_beacon::tests::test_beacon_payload_roundtrip ... ok
# ... all tests pass
```

## References

- [WireGuard Protocol](https://www.wireguard.com/protocol/)
- [PROTOCOL_EVOLUTION_REFINED_FEB_08_2026.md](../../../PROTOCOL_EVOLUTION_REFINED_FEB_08_2026.md)
- [Dark Forest Beacon Implementation](../crates/songbird-discovery/src/dark_forest_beacon.rs)
