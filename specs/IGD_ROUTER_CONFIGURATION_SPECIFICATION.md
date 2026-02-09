# Songbird IGD Router Configuration Specification

**Version**: 1.0  
**Date**: February 8, 2026  
**Status**: IMPLEMENTING  
**Priority**: CRITICAL - #1 blocker for cross-network connectivity  
**Crate**: `songbird-igd`

---

## Overview

Pure Rust implementation of UPnP IGD (RFC 6970) and NAT-PMP (RFC 6886) for automatic router port forwarding. Turns the router from a manual dependency into a tool Songbird configures automatically.

## Problem Statement

Cross-network connectivity between Tower (x86_64, home LAN) and Pixel 8a (aarch64, mobile) fails because the home router does not forward port 3492. All local tests, crypto validation, and beacon exchange pass. The router is the last manual step.

```
Pixel -> hotspot -> carrier NAT -> internet -> home router -> Tower
                                                  ↑
                                        Port 3492 NOT forwarded
```

## Architecture

```
songbird-igd/
├── ssdp.rs       SSDP multicast discovery (UDP 239.255.255.250:1900)
├── soap.rs       SOAP XML control messages (HTTP POST)
├── nat_pmp.rs    NAT-PMP binary protocol (UDP gateway:5351)
├── gateway.rs    Unified abstraction over UPnP/NAT-PMP
├── mapping.rs    Port mapping types and lifecycle
├── renewal.rs    TTL renewal background task
├── error.rs      Typed error handling with SOAP codes
└── lib.rs        Public API and constants
```

## Protocol 1: UPnP IGD (RFC 6970)

### Step 1: SSDP Discovery

UDP multicast M-SEARCH to `239.255.255.250:1900`:

```
M-SEARCH * HTTP/1.1
HOST: 239.255.255.250:1900
MAN: "ssdp:discover"
MX: 3
ST: urn:schemas-upnp-org:device:InternetGatewayDevice:1
```

Also queries `ST: urn:schemas-upnp-org:service:WANIPConnection:1` for routers that only respond to service-level queries.

### Step 2: Device Description

HTTP GET on the LOCATION URL from SSDP response. Parse XML for:
- `serviceType`: WANIPConnection:1
- `controlURL`: SOAP endpoint path

### Step 3: SOAP Control

HTTP POST SOAP envelopes to control URL:

**AddPortMapping**: Maps external port to internal IP:port  
**DeletePortMapping**: Removes a mapping  
**GetExternalIPAddress**: Queries WAN IP from router

## Protocol 2: NAT-PMP (RFC 6886)

Binary UDP protocol to gateway:5351. Simpler fallback for Apple/compatible routers.

**Public IP Request**: 2 bytes (`0x00 0x00`)  
**Port Mapping Request**: 12 bytes (version, opcode, reserved, ports, lifetime)  
**Response**: 16 bytes (version, result, epoch, ports, lifetime)

## Protocol 3: PCP (RFC 6887) - Future

Port Control Protocol (successor to NAT-PMP). Handles IPv6, nested NATs, FILTER options. Reserved for future evolution.

## JSON-RPC Methods

### `igd.discover`
Discover router IGD capabilities. Tries UPnP first, NAT-PMP second. Returns protocol details, capabilities, or clear instructions if nothing found.

### `igd.map_port`
Request port mapping: `{external_port, internal_port, protocol, description, ttl}`. Returns mapped endpoint or SOAP error with suggestions.

### `igd.unmap_port`
Remove port mapping on shutdown. Clean up router configuration.

### `igd.status`
Report all current mappings, gateway state, and renewal schedule.

### `igd.external_ip`
Quick external IP query from router (faster than STUN, no external traffic).

### `igd.auto_configure`
All-in-one: discover gateway, map Songbird port, verify reachability. Returns success with endpoint, or failure with manual instructions and fallback tiers.

## Startup Integration

When `SONGBIRD_IGD_ENABLED=true`:

1. After binding to `:3492`, call `igd.auto_configure`
2. If success: port forwarded, beacon updated with verified external endpoint
3. If failure: log clear message with manual instructions, continue with other tiers
4. Spawn renewal task at half-TTL interval
5. On shutdown: call `igd.unmap_port` to clean up

## Error Handling

When IGD unavailable, provide:
- Gateway diagnostics (IP, reachability, HTTP, admin URL)
- UPnP results (devices found, which are IGD, which aren't)
- NAT-PMP results (sent, responded)
- Manual instructions (steps to configure router)
- Router type detection (AT&T, Netgear, etc.)
- Alternative tier suggestions (IPv6, Onion, STUN, Relay)

## Tier Integration

IGD becomes **Tier 0** - automatic configuration before connection attempts:

| Tier | Protocol | Status |
|------|----------|--------|
| 0 | IGD Auto-Config | NEW - Configure router automatically |
| 1 | IPv6 Direct | Works after IGD enables inbound |
| 2 | Sovereign Onion | Fallback if IGD unavailable |
| 3 | IPv4 Direct | Works after IGD enables inbound |
| 4 | LAN Direct | No IGD needed (same subnet) |
| 5 | STUN Hole-Punch | Fallback for non-symmetric NAT |
| 6 | Family Relay | Fallback when no direct path |
| 7 | DNS Beacon | Discovery layer |
| 8 | External Tunnels | WireGuard, etc. |
| 9 | QUIC | Transport layer |

## Cross-Architecture Notes

| Platform | IGD Use | Notes |
|----------|---------|-------|
| Linux Tower (x86_64) | Full UPnP/NAT-PMP | Primary use case |
| Linux USB (aarch64) | Full UPnP/NAT-PMP | May be different subnet |
| Android Pixel | Detect only | Can't configure carrier NAT, use for diagnostics |

## Deep Debt Compliance

- Pure Rust: Zero C dependencies
- Zero unsafe: `#![forbid(unsafe_code)]`
- From scratch: SSDP + SOAP implemented directly (no external protocol crates)
- Runtime discovery: Default gateway from `/proc/net/route`
- Modern async: Full tokio async/await
- Capability-based: Self-discovers router capabilities

## Dependencies

```toml
[dependencies]
tokio = { version = "1.35", features = ["net", "time", "io-util", "macros", "rt"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tracing = "0.1"
thiserror = "1.0"
anyhow = "1.0"
songbird-types = { path = "../songbird-types" }
```

Zero external protocol crates. SSDP and SOAP implemented from scratch.

## Testing

- Unit tests for all protocol parsers
- Integration tests with mock gateway
- Real gateway tests (optional, AT&T BGW320-505)
- E2E test: Tower IGD → Pixel connectivity

## Success Criteria

1. `igd.discover` correctly identifies UPnP/NAT-PMP capable routers
2. `igd.map_port` successfully forwards port 3492
3. `igd.auto_configure` provides zero-touch deployment
4. Clear manual instructions when auto-config unavailable
5. Port mapping renewal maintains connectivity
6. Graceful cleanup on shutdown

## References

- RFC 6970: UPnP Internet Gateway Device Protocol
- RFC 6886: NAT Port Mapping Protocol (NAT-PMP)
- RFC 6887: Port Control Protocol (PCP)
- biomeOS Integration Team cross-network test results (Feb 8, 2026)

