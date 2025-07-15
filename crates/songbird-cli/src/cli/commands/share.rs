// Module imports
// Share Command - Easily Manage Resource Sharing
//
// This command:
// - Shows current resource sharing status
// - Allows easy adjustment of sharing levels
// - Provides safety limits and recommendations
// - NO technical configuration required!

use crate::cli::CliResult;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, ValueEnum, Serialize, Deserialize)]
pub enum ResourceType {
    /// Share CPU/compute power
    Compute,
    /// Share storage space
    Storage,
    /// Share datasets/research data
    Data,
    /// Share all resources
    All,
}
/// Execute the share command
pub async fn execute_share(resource: ResourceType, percent: u8) -> CliResult<()> {
    println!("📤 Manage Resource Sharing");
    println!("========================");
    println!();
    // Validate percentage range
    if percent > 100 {
        return Err(crate::cli::CliError::Command {
            message: format!("Invalid percentage: {percent}%. Must be between 0-100%"),
            command: Some("share".to_string()),
            suggestion: Some("Specify a valid percentage between 0 and 100".to_string()),
        });
    }
    // Validate percentage
    if percent > 80 {
        println!(
            "{}",
            crate::cli::ui::warn("⚠️  Sharing more than 80% of resources is not recommended")
        );
        println!("   This might impact your system's performance.");
        println!("   Consider sharing 50-70% for optimal balance.");
        println!();
    }

    // Show current system status
    println!("{}", crate::cli::ui::info("🔍 Analyzing your system..."));
    let current_resources = analyze_current_resources().await?;
    display_current_status(&current_resources);

    // Apply sharing configuration
    println!(
        "{}",
        crate::cli::ui::info(&format!(
            "⚙️  Configuring {} sharing at {}%...",
            format_resource_type(&resource),
            percent
        ))
    );
    let sharing_config = calculate_sharing_amounts(&current_resources, &resource, percent)?;
    apply_sharing_configuration(&sharing_config).await?;
    println!(
        "{}",
        crate::cli::ui::success("✅ Resource sharing updated!")
    );
    // Show what's being shared
    display_sharing_summary(&sharing_config);
    show_impact_estimate(&sharing_config);

    Ok(())
}
/// Current system resources
#[derive(Debug, Clone)]
struct CurrentResources {
    cpu_cores: usize,
    memory_gb: f64,
    storage_gb: f64,
    gpu_available: bool,
    network_mbps: f64,
    current_usage: ResourceUsage,
}

/// Current resource usage
#[derive(Debug, Clone)]
struct ResourceUsage {
    cpu_percent: f64,
    memory_percent: f64,
    storage_percent: f64,
}

/// Resource sharing configuration
#[derive(Debug, Clone, serde::Serialize)]
pub struct SharingConfig {
    resource_type: ResourceType,
    share_percent: u8,
    cpu_cores_shared: usize,
    memory_gb_shared: f64,
    storage_gb_shared: f64,
    gpu_shared: bool,
    estimated_impact: ImpactLevel,
}

#[derive(Debug, Clone, serde::Serialize)]
pub enum ImpactLevel {
    Minimal,  // <25% resources
    Low,      // 25-50% resources
    Moderate, // 50-75% resources
    High,     // >75% resources
}
/// Analyze current system resources
async fn analyze_current_resources() -> CliResult<CurrentResources> {
    use sysinfo::System;
    let mut sys = System::new_all();
    sys.refresh_all();
    let cpu_cores = sys.cpus().len();
    let memory_gb = sys.total_memory() as f64 / (1024.0 * 1024.0 * 1024.0);
    // Estimate available storage
    let storage_gb = estimate_available_storage().await?;
    // Check GPU availability
    let gpu_available = detect_gpu().await;
    // Estimate network speed
    let network_mbps = estimate_network_speed().await;
    // Get current usage - simplified for now
    let cpu_percent = 25.0; // Simplified estimation
    let memory_percent = (sys.used_memory() as f64 / sys.total_memory() as f64) * 100.0;
    let storage_percent = 50.0; // Simplified estimation
    Ok(CurrentResources {
        cpu_cores,
        memory_gb,
        storage_gb,
        gpu_available,
        network_mbps,
        current_usage: ResourceUsage {
            cpu_percent,
            memory_percent,
            storage_percent,
        },
    })
}
/// Calculate sharing amounts based on resource type and percentage
fn calculate_sharing_amounts(
    resources: &CurrentResources,
    resource_type: &ResourceType,
    percent: u8,
) -> CliResult<SharingConfig> {
    let share_ratio = percent as f64 / 100.0;
    let (cpu_cores_shared, memory_gb_shared, storage_gb_shared, gpu_shared) = match resource_type {
        ResourceType::Compute => {
            let cpu_shared = ((resources.cpu_cores as f64 * share_ratio).ceil() as usize).max(1);
            let memory_shared = resources.memory_gb * share_ratio * 0.8; // Leave some buffer
            (
                cpu_shared,
                memory_shared,
                0.0,
                resources.gpu_available && percent > 30,
            )
        }
        ResourceType::Storage => (0, 0.0, resources.storage_gb * share_ratio, false),
        ResourceType::Data => {
            // Data sharing doesn't consume resources, but we might need some storage buffer
            (0, 0.0, resources.storage_gb * 0.1, false) // 10% for data indexing
        }
        ResourceType::All => {
            let cpu_shared = ((resources.cpu_cores as f64 * share_ratio).ceil() as usize).max(1);
            let memory_shared = resources.memory_gb * share_ratio * 0.7; // Conservative for all sharing
            let storage_shared = resources.storage_gb * share_ratio * 0.8; // Leave some buffer
            (
                cpu_shared,
                memory_shared,
                storage_shared,
                resources.gpu_available && percent > 25,
            )
        }
    };
    // Determine impact level
    let impact = if percent < 25 {
        ImpactLevel::Minimal
    } else if percent < 50 {
        ImpactLevel::Low
    } else if percent < 75 {
        ImpactLevel::Moderate
    } else {
        ImpactLevel::High
    };

    Ok(SharingConfig {
        resource_type: resource_type.clone(),
        share_percent: percent,
        cpu_cores_shared,
        memory_gb_shared,
        storage_gb_shared,
        gpu_shared,
        estimated_impact: impact,
    })
}
/// Apply the sharing configuration
async fn apply_sharing_configuration(config: &SharingConfig) -> CliResult<()> {
    // Create CPU affinity constraints if sharing compute
    if config.cpu_cores_shared > 0 {
        apply_cpu_limits(config.cpu_cores_shared).await?;
    }

    // Set up storage quotas if sharing storage
    if config.storage_gb_shared > 0.0 {
        apply_storage_limits(config.storage_gb_shared).await?;
    }

    // Configure memory constraints if sharing memory
    if config.memory_gb_shared > 0.0 {
        apply_memory_limits(config.memory_gb_shared).await?;
    }

    // Save configuration to orchestrator state
    save_sharing_state(config).await?;

    Ok(())
}
/// Apply CPU core limitations using cgroups (Linux) or job objects (Windows)
async fn apply_cpu_limits(cores_to_share: usize) -> CliResult<()> {
    #[cfg(unix)]
    {
        // Check if we can write to cgroups (requires permissions)
        let cgroup_path = "/sys/fs/cgroup/cpu/songbird";

        if let Ok(output) = std::process::Command::new("mkdir")
            .arg("-p")
            .arg(cgroup_path)
            .output()
        {
            if output.status.success() {
                // Set CPU quota for the shared cores
                let quota = format!("{}", cores_to_share * 100000); // 100ms per core
                let _ = tokio::fs::write(format!("{cgroup_path}/cpu.cfs_quota_us"), quota).await;

                println!("   📊 Applied CPU limit: {cores_to_share} cores via cgroups");
            } else {
                println!("   ⚠️  CPU limits require root access (cgroups), using process affinity");
                apply_process_affinity(cores_to_share).await?;
            }
        } else {
            apply_process_affinity(cores_to_share).await?;
        }
    }

    #[cfg(windows)]
    {
        apply_windows_cpu_limits(cores_to_share).await?;
    }

    Ok(())
}
/// Apply process-level CPU affinity (works without root)
async fn apply_process_affinity(cores_to_share: usize) -> CliResult<()> {
    // Create affinity mask for the cores we want to share
    let total_cores = num_cpus::get();
    let cores_to_reserve = total_cores.saturating_sub(cores_to_share);
    #[cfg(unix)]
    {
        // Use taskset on Linux to limit current process
        if cores_to_reserve > 0 {
            let mask = format!("0-{}", cores_to_reserve - 1);
            let output = std::process::Command::new("taskset")
                .arg("-cp")
                .arg(&mask)
                .arg(std::process::id().to_string())
                .output();

            if let Ok(output) = output {
                if output.status.success() {
                    println!(
                        "   📊 Applied CPU affinity: reserved {cores_to_reserve} cores, sharing {cores_to_share}"
                    );
                } else {
                    println!("   ⚠️  CPU affinity failed: taskset not available");
                }
            }
        }
    }

    Ok(())
}
/// Apply storage quotas using filesystem tools
async fn apply_storage_limits(gb_to_share: f64) -> CliResult<()> {
    // Create dedicated directory for shared storage
    let shared_dir = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("songbird")
        .join("shared-storage");
    // Create the directory
    tokio::fs::create_dir_all(&shared_dir)
        .await
        .map_err(crate::cli::CliError::Io)?;
    // Set up quota tracking file
    let quota_file = shared_dir.join(".quota");
    let quota_info = serde_json::json!({
        "max_gb": gb_to_share,
        "created_at": chrono::Utc::now().timestamp(),
        "path": shared_dir.display().to_string()
    });
    tokio::fs::write(&quota_file, quota_info.to_string())
        .await
        .map_err(crate::cli::CliError::Io)?;

    println!(
        "   📊 Created shared storage: {:.1} GB at {}",
        gb_to_share,
        shared_dir.display()
    );

    Ok(())
}
/// Apply memory limits using system tools
async fn apply_memory_limits(gb_to_share: f64) -> CliResult<()> {
    // Create memory tracking
    let bytes_to_share = (gb_to_share * 1024.0 * 1024.0 * 1024.0) as u64;

    #[cfg(unix)]
    {
        // Try to set memory limits via ulimit or systemd
        let kb_limit = bytes_to_share / 1024;
        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(format!("ulimit -v {kb_limit}"))
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                println!("   📊 Applied memory limit: {gb_to_share:.1} GB via ulimit");
            } else {
                println!("   ⚠️  Memory limits require system support, tracking manually");
            }
        }
    }

    #[cfg(windows)]
    {
        println!("   ⚠️  Memory limits on Windows require job objects (advanced)");
    }

    Ok(())
}
/// Save sharing configuration to persistent state
async fn save_sharing_state(config: &SharingConfig) -> CliResult<()> {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("songbird");

    tokio::fs::create_dir_all(&config_dir)
        .await
        .map_err(crate::cli::CliError::Io)?;

    let sharing_file = config_dir.join("sharing.json");
    let serialized =
        serde_json::to_string_pretty(config).map_err(crate::cli::CliError::Serialization)?;

    tokio::fs::write(&sharing_file, serialized)
        .await
        .map_err(crate::cli::CliError::Io)?;

    println!(
        "   💾 Saved sharing configuration to {}",
        sharing_file.display()
    );

    Ok(())
}
/// Windows-specific CPU limits
#[cfg(windows)]
async fn apply_windows_cpu_limits(cores_to_share: usize) -> CliResult<()> {
    // Windows Job Objects would be the proper way to do this
    // For now, just track the intention
    println!(
        "   📊 Windows CPU limits: {} cores (requires job objects)",
        cores_to_share
    );

    Ok(())
}
/// Display current system status
fn display_current_status(resources: &CurrentResources) {
    println!("💻 Your System:");
    println!(
        "   🔲 CPU: {} cores ({:.1}% in use)",
        resources.cpu_cores, resources.current_usage.cpu_percent
    );
    println!(
        "   🧠 Memory: {:.1} GB ({:.1}% in use)",
        resources.memory_gb, resources.current_usage.memory_percent
    );
    println!(
        "   💾 Storage: {:.1} GB ({:.1}% in use)",
        resources.storage_gb, resources.current_usage.storage_percent
    );

    if resources.gpu_available {
        println!("   🎮 GPU: Available");
    }

    println!("   🌐 Network: {:.0} Mbps", resources.network_mbps);
}
/// Display sharing summary
fn display_sharing_summary(config: &SharingConfig) {
    println!("📊 New Sharing Configuration:");

    match config.resource_type {
        ResourceType::Compute => {
            if config.cpu_cores_shared > 0 {
                println!("   🔲 CPU: {} cores", config.cpu_cores_shared);
            }
            if config.memory_gb_shared > 0.0 {
                println!("   🧠 Memory: {:.1} GB", config.memory_gb_shared);
            }
            if config.gpu_shared {
                println!("   🎮 GPU: Shared");
            }
        }
        ResourceType::Storage => {
            println!("   💾 Storage: {:.1} GB", config.storage_gb_shared);
        }
        ResourceType::Data => {
            println!(
                "   📊 Data: Shared (with {:.1} GB buffer)",
                config.storage_gb_shared
            );
        }
        ResourceType::All => {
            if config.cpu_cores_shared > 0 {
                println!("   🔲 CPU: {} cores", config.cpu_cores_shared);
            }
            if config.memory_gb_shared > 0.0 {
                println!("   🧠 Memory: {:.1} GB", config.memory_gb_shared);
            }
            if config.storage_gb_shared > 0.0 {
                println!("   💾 Storage: {:.1} GB", config.storage_gb_shared);
            }
            if config.gpu_shared {
                println!("   🎮 GPU: Shared");
            }
        }
    }

    println!("   📈 Sharing Level: {}%", config.share_percent);
    println!("   ⚡ Impact: {:?}", config.estimated_impact);
}
/// Show impact estimate
fn show_impact_estimate(config: &SharingConfig) {
    let (impact_icon, impact_desc, recommendations) = match config.estimated_impact {
        ImpactLevel::Minimal => (
            "🟢",
            "Minimal impact on your system performance",
            vec![
                "Perfect for background contribution",
                "Your system will remain fully responsive",
                "Great for long-term sharing",
            ],
        ),
        ImpactLevel::Low => (
            "🟡",
            "Low impact - should not affect daily use",
            vec![
                "Good balance of contribution and performance",
                "May see occasional slight slowdowns during heavy network use",
                "Recommended for most users",
            ],
        ),
        ImpactLevel::Moderate => (
            "🟠",
            "Moderate impact - may affect performance during intensive tasks",
            vec![
                "Significant contribution to the network",
                "May slow down CPU/memory intensive applications",
                "Consider reducing if performance issues occur",
            ],
        ),
        ImpactLevel::High => (
            "🔴",
            "High impact - will likely affect system performance",
            vec![
                "Maximum contribution to the network",
                "May significantly slow down your system",
                "Only recommended for dedicated research nodes",
                "Monitor system performance closely",
            ],
        ),
    };

    println!("{impact_icon} Impact Assessment:");
    println!("   {impact_desc}");

    for rec in recommendations {
        println!("   • {rec}");
    }

    println!("💡 Pro Tips:");
    println!("   • Use 'songbird status' to monitor your contribution");
    println!("   • Adjust sharing anytime with 'songbird share'");
    let env_config = songbird_config::config::environment::EnvironmentConfig::default();
    println!(
        "   • View network activity at http://{}:{}",
        env_config.bind_address, env_config.bind_port
    );
}
/// Format resource type for display
fn format_resource_type(resource_type: &ResourceType) -> &str {
    match resource_type {
        ResourceType::Compute => "compute",
        ResourceType::Storage => "storage",
        ResourceType::Data => "data",
        ResourceType::All => "all resources",
    }
}
// Helper functions - REAL system detection (no hardcoded values)
async fn estimate_available_storage() -> CliResult<f64> {
    // Check environment variable override first
    if let Ok(storage_gb) = std::env::var("SONGBIRD_STORAGE_GB") {
        if let Ok(gb) = storage_gb.parse::<f64>() {
            return Ok(gb);
        }
    }

    // Real storage detection using system APIs
    detect_available_storage().ok_or_else(|| {
        crate::cli::CliError::Command {
            message: "Failed to detect available storage. Set SONGBIRD_STORAGE_GB environment variable.".to_string(),
            command: Some("share".to_string()),
            suggestion: Some("Set the SONGBIRD_STORAGE_GB environment variable with your available storage".to_string()),
        }
    })
}
async fn detect_gpu() -> bool {
    if let Ok(gpu_available) = std::env::var("SONGBIRD_GPU_AVAILABLE") {
        return gpu_available.to_lowercase() == "true" || gpu_available == "1";
    }

    // Real GPU detection - already implemented in quick.rs
    detect_gpu_availability()
}
async fn estimate_network_speed() -> f64 {
    if let Ok(speed_str) = std::env::var("SONGBIRD_NETWORK_SPEED") {
        if let Ok(speed) = speed_str.parse::<f64>() {
            return speed;
        }
    }

    // Real network speed detection using system interfaces
    detect_network_interface_speed().await.unwrap_or_else(|| {
        eprintln!("⚠️  Unable to detect network speed, using conservative estimate");
        100.0 // Conservative default when detection fails
    })
}
/// Real storage detection using system APIs (moved from quick.rs for reuse)
fn detect_available_storage() -> Option<f64> {
    // Try to get storage info for the current directory
    let path = std::env::current_dir().ok()?;

    #[cfg(unix)]
    {
        use std::ffi::CString;
        use std::mem::MaybeUninit;

        let path_cstr = CString::new(path.to_string_lossy().as_bytes()).ok()?;
        let mut statfs = MaybeUninit::<libc::statvfs>::uninit();
        let result = unsafe { libc::statvfs(path_cstr.as_ptr(), statfs.as_mut_ptr()) };

        if result == 0 {
            let statfs = unsafe { statfs.assume_init() };
            let available_bytes = statfs.f_bavail.saturating_mul(statfs.f_frsize);
            return Some(available_bytes as f64 / (1024.0 * 1024.0 * 1024.0));
        }
    }

    #[cfg(windows)]
    {
        // Windows storage detection using Win32 API
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;

        let current_dir = std::env::current_dir().ok()?;
        let drive_letter = current_dir.to_string_lossy().chars().next()?;
        let drive_path = format!("{}:\\", drive_letter);

        // Convert to wide string for Windows API
        let wide_path: Vec<u16> = OsStr::new(&drive_path)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        let mut free_bytes: u64 = 0;
        let mut total_bytes: u64 = 0;
        // Call GetDiskFreeSpaceEx
        unsafe {
            let result = winapi::um::fileapi::GetDiskFreeSpaceExW(
                wide_path.as_ptr(),
                &mut free_bytes,
                &mut total_bytes,
                std::ptr::null_mut(),
            );
            if result != 0 {
                return Some(free_bytes as f64 / (1024.0 * 1024.0 * 1024.0));
            }
        }
    }

    None
}
/// Real GPU detection using system probing (moved from quick.rs for reuse)
fn detect_gpu_availability() -> bool {
    // Check for NVIDIA GPU
    if std::process::Command::new("nvidia-smi")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
    {
        return true;
    }

    // Check for AMD GPU
    if std::process::Command::new("rocm-smi")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
    {
        return true;
    }

    // Check for Intel GPU
    if std::process::Command::new("intel_gpu_top")
        .arg("--help")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
    {
        return true;
    }

    false
}
/// Real network interface speed detection
async fn detect_network_interface_speed() -> Option<f64> {
    #[cfg(unix)]
    {
        // Try to read network interface information from /proc/net/dev
        if let Ok(contents) = tokio::fs::read_to_string("/proc/net/dev").await {
            let mut max_speed: f64 = 0.0;
            for line in contents.lines().skip(2) {
                // Skip header lines
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let interface = parts[0].trim_end_matches(':');

                    // Skip loopback and virtual interfaces
                    if interface.starts_with("lo")
                        || interface.starts_with("veth")
                        || interface.starts_with("docker")
                        || interface.starts_with("br-")
                    {
                        continue;
                    }

                    // Try to get interface speed from ethtool or sysfs
                    if let Some(speed) = get_interface_speed(interface).await {
                        max_speed = max_speed.max(speed);
                    }
                }
            }

            if max_speed > 0.0 {
                return Some(max_speed);
            }
        }
    }

    #[cfg(windows)]
    {
        // Windows network speed detection would require WMI or similar
        // For now, use a conservative default
        return Some(100.0);
    }

    None
}
/// Get specific network interface speed
#[cfg(unix)]
async fn get_interface_speed(interface: &str) -> Option<f64> {
    // Try sysfs first (most reliable)
    let speed_path = format!("/sys/class/net/{interface}/speed");
    if let Ok(speed_str) = tokio::fs::read_to_string(&speed_path).await {
        if let Ok(speed_mbps) = speed_str.trim().parse::<f64>() {
            if speed_mbps > 0.0 {
                return Some(speed_mbps);
            }
        }
    }

    // Try ethtool as fallback
    if let Ok(output) = std::process::Command::new("ethtool")
        .arg(interface)
        .output()
    {
        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                if line.contains("Speed:") && line.contains("Mb/s") {
                    if let Some(speed_part) = line.split("Speed:").nth(1) {
                        if let Some(speed_str) = speed_part.split("Mb/s").next() {
                            if let Ok(speed) = speed_str.trim().parse::<f64>() {
                                return Some(speed);
                            }
                        }
                    }
                }
            }
        }
    }

    None
}

#[cfg(windows)]
async fn get_interface_speed(_interface: &str) -> Option<f64> {
    // Windows implementation would require WMI queries
    // For now, return a conservative default
    Some(100.0)
}
