// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Node capability discovery for deployment endpoints.

use axum::{Json, extract::State};
use songbird_types::{network_info, sys_metrics};
use tracing::info;

use super::types::{
    BandwidthEstimate, ChunkedUploadMethod, DeploymentCapabilities, DeploymentMethods,
    DeploymentPreferences, DeploymentState, NetworkCapabilities, ResourceInfo, SingleUploadMethod,
    StreamingUploadMethod,
};

/// GET /api/deployment/capabilities - Discover node capabilities
pub async fn get_capabilities(
    State(state): State<DeploymentState>,
) -> Json<DeploymentCapabilities> {
    info!("📊 Capability discovery request received");

    let network_type = detect_network_type();
    let bandwidth = estimate_bandwidth(&network_type);

    let mem = sys_metrics::memory_info().unwrap_or(sys_metrics::MemoryInfo {
        total: 0,
        available: 0,
    });
    let _total_memory = mem.total_gb();
    let available_memory = mem.available / (1024 * 1024 * 1024);
    let cpu_cores = std::thread::available_parallelism().map_or(1, std::num::NonZero::get);
    let cpu_load = sys_metrics::load_percent();

    let available_storage = sys_metrics::total_disk_gb().unwrap_or(0) as u64;

    let current_deployments = state.deployments.read().await.len();

    let max_concurrent = calculate_max_concurrent(available_memory);

    let capabilities = DeploymentCapabilities {
        node_id: gethostname::gethostname()
            .into_string()
            .unwrap_or_else(|_| String::from("unknown")),
        timestamp: chrono::Utc::now().to_rfc3339(),
        network: NetworkCapabilities {
            network_type: network_type.clone(),
            bandwidth_estimate: bandwidth,
        },
        deployment_methods: DeploymentMethods {
            single: SingleUploadMethod {
                enabled: true,
                max_size_mb: 50,
                compression_supported: vec![String::from("gzip")],
                recommended_for: String::from("< 10MB"),
            },
            chunked: ChunkedUploadMethod {
                enabled: true,
                max_total_size_mb: 1000,
                chunk_size_mb: 10,
                max_chunks: 100,
                compression_supported: vec![String::from("gzip")],
                recommended_for: String::from("2MB - 500MB"),
            },
            streaming: StreamingUploadMethod {
                enabled: false,
                unlimited: true,
                compression_supported: vec![String::from("gzip")],
                recommended_for: String::from("> 500MB"),
            },
        },
        resources: ResourceInfo {
            available_storage_gb: available_storage,
            available_memory_gb: available_memory,
            cpu_cores,
            cpu_load_percent: cpu_load,
            max_concurrent_deployments: max_concurrent,
            current_deployments,
        },
        preferences: DeploymentPreferences {
            preferred_compression: String::from("gzip"),
            preferred_method: select_preferred_method(&network_type),
            encryption_required: network_type == "internet",
        },
    };

    info!(
        "✅ Capabilities: {} network, {}MB up/down, {}GB available storage",
        capabilities.network.network_type,
        capabilities.network.bandwidth_estimate.upload_mbps,
        capabilities.resources.available_storage_gb
    );

    Json(capabilities)
}

/// Detect network type (LAN, VPN, or Internet) based on environment and interface heuristics.
pub fn detect_network_type() -> String {
    if songbird_process_env::var("SONGBIRD_NETWORK_TYPE").is_ok_and(|v| !v.is_empty()) {
        return songbird_process_env::var("SONGBIRD_NETWORK_TYPE")
            .unwrap_or_else(|_| String::from("lan"));
    }

    if is_vpn_likely() {
        return String::from("vpn");
    }

    if is_internet_facing() {
        return String::from("internet");
    }

    String::from("lan")
}

fn is_vpn_likely() -> bool {
    songbird_process_env::var("SONGBIRD_VPN_ACTIVE")
        .is_ok_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        || tun_interface_exists()
}

fn tun_interface_exists() -> bool {
    std::fs::read_dir("/sys/class/net")
        .map(|entries| {
            entries.filter_map(Result::ok).any(|e| {
                let name = e.file_name();
                let n = name.to_string_lossy();
                n.starts_with("tun") || n.starts_with("wg") || n.starts_with("tailscale")
            })
        })
        .unwrap_or(false)
}

fn is_internet_facing() -> bool {
    let bind_addr =
        songbird_process_env::var("SONGBIRD_PRODUCTION_BIND_ADDRESS").unwrap_or_default();
    if bind_addr == songbird_types::constants::PRODUCTION_BIND_ADDRESS
        && network_info::has_public_ipv4_interface()
    {
        return true;
    }

    songbird_process_env::var("SONGBIRD_FEDERATION_MODE")
        .is_ok_and(|v| v.eq_ignore_ascii_case("internet") || v.eq_ignore_ascii_case("wan"))
}

/// Peer-aware network type classification.
///
/// Classifies the network relationship between this node and a specific peer IP.
/// Returns `"lan"` / `"vpn"` / `"internet"` based on IP analysis:
/// - Same subnet as a local interface → `"lan"`
/// - Private IP but different subnet → `"vpn"` (cross-subnet or VPN)
/// - Public IP → `"internet"`
pub fn detect_network_type_for_peer(peer_ip: &str) -> String {
    if let Ok(forced) = songbird_process_env::var("SONGBIRD_NETWORK_TYPE")
        && !forced.is_empty()
    {
        return forced;
    }

    let Some(peer_octets) = network_info::parse_ipv4_octets(peer_ip) else {
        return detect_network_type();
    };

    if peer_octets[0] == 127 {
        return String::from("lan");
    }

    let local_addrs: Vec<[u8; 4]> =
        network_info::local_ipv4_from_fib_trie().into_iter().map(|ip| ip.octets()).collect();
    if local_addrs.iter().any(|local| network_info::same_subnet_24(*local, peer_octets)) {
        return String::from("lan");
    }

    if network_info::is_private_or_special(peer_octets) {
        if tun_interface_exists() {
            return String::from("vpn");
        }
        return String::from("vpn");
    }

    String::from("internet")
}

/// Estimate bandwidth based on network type
pub fn estimate_bandwidth(network_type: &str) -> BandwidthEstimate {
    match network_type {
        "lan" => BandwidthEstimate {
            download_mbps: 1000,
            upload_mbps: 1000,
            latency_ms: 1,
            confidence: String::from("high"),
        },
        "vpn" => BandwidthEstimate {
            download_mbps: 100,
            upload_mbps: 100,
            latency_ms: 10,
            confidence: String::from("medium"),
        },
        _ => BandwidthEstimate {
            download_mbps: 50,
            upload_mbps: 20,
            latency_ms: 50,
            confidence: String::from("low"),
        },
    }
}

/// Select preferred deployment method based on network conditions.
fn select_preferred_method(network_type: &str) -> String {
    match network_type {
        "lan" => String::from("single"),
        "vpn" => String::from("chunked"),
        "internet" => String::from("chunked"),
        _ => String::from("single"),
    }
}

/// Calculate max concurrent deployments based on available memory
pub fn calculate_max_concurrent(available_memory_gb: u64) -> usize {
    #[expect(
        clippy::cast_possible_truncation,
        reason = "truncation acceptable: GB count bounded; only used as deployment concurrency hint"
    )]
    let memory_as_usize = available_memory_gb as usize;
    memory_as_usize.clamp(1, 10)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "test assertions")]
mod tests {
    use super::*;
    use songbird_process_env::ScopedEnv;

    #[test]
    fn parse_ipv4_octets_valid() {
        assert_eq!(network_info::parse_ipv4_octets("192.168.1.144"), Some([192, 168, 1, 144]));
        assert_eq!(network_info::parse_ipv4_octets("10.0.0.1"), Some([10, 0, 0, 1]));
        assert_eq!(network_info::parse_ipv4_octets("255.255.255.255"), Some([255, 255, 255, 255]));
    }

    #[test]
    fn parse_ipv4_octets_invalid() {
        assert_eq!(network_info::parse_ipv4_octets("not-an-ip"), None);
        assert_eq!(network_info::parse_ipv4_octets("192.168.1"), None);
        assert_eq!(network_info::parse_ipv4_octets(""), None);
    }

    #[test]
    fn is_private_or_special_classifies_correctly() {
        assert!(network_info::is_private_or_special([10, 0, 0, 1]));
        assert!(network_info::is_private_or_special([172, 16, 0, 1]));
        assert!(network_info::is_private_or_special([172, 31, 255, 255]));
        assert!(network_info::is_private_or_special([192, 168, 1, 1]));
        assert!(network_info::is_private_or_special([127, 0, 0, 1]));
        assert!(network_info::is_private_or_special([169, 254, 1, 1]));
        assert!(!network_info::is_private_or_special([8, 8, 8, 8]));
        assert!(!network_info::is_private_or_special([157, 230, 3, 183]));
        assert!(!network_info::is_private_or_special([172, 32, 0, 1]));
    }

    #[test]
    fn same_subnet_24_works() {
        assert!(network_info::same_subnet_24([192, 168, 1, 1], [192, 168, 1, 254]));
        assert!(!network_info::same_subnet_24([192, 168, 1, 1], [192, 168, 2, 1]));
        assert!(!network_info::same_subnet_24([10, 0, 0, 1], [10, 0, 1, 1]));
    }

    #[test]
    fn peer_loopback_is_lan() {
        let _lock = songbird_process_env::test_env_lock();
        songbird_process_env::remove_var("SONGBIRD_NETWORK_TYPE");
        assert_eq!(detect_network_type_for_peer("127.0.0.1"), "lan");
    }

    #[test]
    fn peer_public_ip_is_internet() {
        let _lock = songbird_process_env::test_env_lock();
        songbird_process_env::remove_var("SONGBIRD_NETWORK_TYPE");
        assert_eq!(detect_network_type_for_peer("157.230.3.183"), "internet");
        assert_eq!(detect_network_type_for_peer("8.8.8.8"), "internet");
    }

    #[test]
    fn env_override_applies_to_peer_detection() {
        let _lock = songbird_process_env::test_env_lock();
        let _env = ScopedEnv::new("SONGBIRD_NETWORK_TYPE", "vpn");
        assert_eq!(detect_network_type_for_peer("8.8.8.8"), "vpn");
    }

    #[test]
    fn invalid_peer_ip_falls_back_to_node_detection() {
        let _lock = songbird_process_env::test_env_lock();
        songbird_process_env::remove_var("SONGBIRD_NETWORK_TYPE");
        let result = detect_network_type_for_peer("not-an-ip");
        assert!(["lan", "vpn", "internet"].contains(&result.as_str()));
    }

    #[test]
    fn private_peer_not_on_local_subnet_is_vpn() {
        let _lock = songbird_process_env::test_env_lock();
        songbird_process_env::remove_var("SONGBIRD_NETWORK_TYPE");
        // 172.16.x.x is private — unless this machine has a 172.16.x.0/24 interface,
        // it should classify as vpn
        let result = detect_network_type_for_peer("172.16.99.99");
        assert!(result == "vpn" || result == "lan");
    }

    #[test]
    fn calculate_max_concurrent_clamps() {
        assert_eq!(calculate_max_concurrent(0), 1);
        assert_eq!(calculate_max_concurrent(1), 1);
        assert_eq!(calculate_max_concurrent(5), 5);
        assert_eq!(calculate_max_concurrent(10), 10);
        assert_eq!(calculate_max_concurrent(100), 10);
    }

    #[test]
    fn bandwidth_estimates_reasonable() {
        let lan = estimate_bandwidth("lan");
        assert_eq!(lan.download_mbps, 1000);
        assert_eq!(lan.latency_ms, 1);

        let vpn = estimate_bandwidth("vpn");
        assert_eq!(vpn.download_mbps, 100);

        let internet = estimate_bandwidth("internet");
        assert_eq!(internet.download_mbps, 50);
    }

    #[test]
    fn detect_network_type_env_override() {
        let _lock = songbird_process_env::test_env_lock();
        let _env = ScopedEnv::new("SONGBIRD_NETWORK_TYPE", "internet");
        assert_eq!(detect_network_type(), "internet");
    }
}
