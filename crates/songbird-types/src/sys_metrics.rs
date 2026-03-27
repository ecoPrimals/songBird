// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Pure Rust system metrics — replaces `sysinfo` crate for ecoBin v3.0 compliance.
//!
//! Reads directly from `/proc` and `/sys` on Linux. Returns `None` on unsupported
//! platforms rather than pulling in `rayon` + `crossbeam-*` via `sysinfo`.
//!
//! ## Coverage
//!
//! | Metric | Source | `sysinfo` equivalent |
//! |--------|--------|---------------------|
//! | Total memory | `/proc/meminfo` | `System::total_memory()` |
//! | Available memory | `/proc/meminfo` | `System::available_memory()` |
//! | Disk total | `/sys/block/*/size` | `Disks::total_space()` sum |

const BYTES_PER_GB: u64 = 1024 * 1024 * 1024;
const KB_TO_BYTES: u64 = 1024;

/// System memory information in bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoryInfo {
    /// Total physical memory in bytes.
    pub total: u64,
    /// Available memory in bytes (kernel estimate of reclaimable + free).
    pub available: u64,
}

impl MemoryInfo {
    /// Used memory in bytes.
    #[must_use]
    pub const fn used(&self) -> u64 {
        self.total.saturating_sub(self.available)
    }

    /// Total memory in gigabytes (integer, rounded down).
    #[must_use]
    pub const fn total_gb(&self) -> u64 {
        self.total / BYTES_PER_GB
    }

    /// Available memory in gigabytes (floating point).
    #[must_use]
    #[expect(clippy::cast_precision_loss, reason = "GB-scale values fit f64 mantissa")]
    pub fn available_gb(&self) -> f64 {
        self.available as f64 / BYTES_PER_GB as f64
    }

    /// Memory usage as a percentage (0.0-100.0).
    #[must_use]
    #[expect(clippy::cast_precision_loss, reason = "byte counts in practical range fit f64")]
    pub fn usage_percent(&self) -> f64 {
        if self.total == 0 {
            return 0.0;
        }
        (self.used() as f64 / self.total as f64) * 100.0
    }
}

/// Read memory info from `/proc/meminfo`.
///
/// Returns `None` on non-Linux platforms or if the file cannot be read.
#[cfg(target_os = "linux")]
pub fn memory_info() -> Option<MemoryInfo> {
    let contents = std::fs::read_to_string("/proc/meminfo").ok()?;
    parse_meminfo(&contents)
}

/// Non-Linux fallback.
#[cfg(not(target_os = "linux"))]
pub fn memory_info() -> Option<MemoryInfo> {
    None
}

/// Total physical memory in bytes (0 if unavailable).
#[must_use]
pub fn total_memory_bytes() -> u64 {
    memory_info().map_or(0, |m| m.total)
}

/// Total physical memory in gigabytes (integer).
#[must_use]
#[expect(clippy::cast_possible_truncation, reason = "memory GB always fits usize")]
pub fn total_memory_gb() -> usize {
    memory_info().map_or(0, |m| m.total_gb() as usize)
}

/// Disk space information for a block device.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiskInfo {
    /// Device name (e.g., "sda", "nvme0n1").
    pub device: String,
    /// Total space in bytes.
    pub total_bytes: u64,
}

impl DiskInfo {
    /// Total space in gigabytes.
    #[must_use]
    #[expect(clippy::cast_precision_loss, reason = "disk GB values fit f64 mantissa")]
    pub fn total_gb(&self) -> f64 {
        self.total_bytes as f64 / BYTES_PER_GB as f64
    }
}

/// Read disk info from `/sys/block/*/size` for real block devices.
///
/// Filters to physical devices (sd*, nvme*, vd*, xvd*, hd*) and excludes
/// partitions, loop devices, and virtual devices.
#[cfg(target_os = "linux")]
pub fn disk_info() -> Vec<DiskInfo> {
    let Ok(entries) = std::fs::read_dir("/sys/block") else {
        return Vec::new();
    };

    let mut disks = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        let is_physical = name_str.starts_with("sd")
            || name_str.starts_with("nvme")
            || name_str.starts_with("vd")
            || name_str.starts_with("xvd")
            || name_str.starts_with("hd");

        if !is_physical {
            continue;
        }

        let size_path = entry.path().join("size");
        if let Ok(size_str) = std::fs::read_to_string(&size_path)
            && let Ok(sectors) = size_str.trim().parse::<u64>()
        {
            let total_bytes = sectors * 512;
            if total_bytes > 0 {
                disks.push(DiskInfo {
                    device: name_str.into_owned(),
                    total_bytes,
                });
            }
        }
    }

    disks
}

/// Non-Linux fallback.
#[cfg(not(target_os = "linux"))]
pub fn disk_info() -> Vec<DiskInfo> {
    Vec::new()
}

/// Total disk space across all physical block devices in gigabytes.
#[must_use]
#[allow(clippy::cast_possible_truncation, reason = "disk GB always fits usize")]
pub fn total_disk_gb() -> Option<usize> {
    let disks = disk_info();
    if disks.is_empty() {
        return None;
    }
    let total: u64 = disks.iter().map(|d| d.total_bytes).sum();
    Some((total / BYTES_PER_GB) as usize)
}

// ---- Internal parsers ----

fn parse_meminfo(contents: &str) -> Option<MemoryInfo> {
    let mut total = None;
    let mut available = None;

    for line in contents.lines() {
        if let Some(rest) = line.strip_prefix("MemTotal:") {
            total = parse_kb_value(rest);
        } else if let Some(rest) = line.strip_prefix("MemAvailable:") {
            available = parse_kb_value(rest);
        }
        if total.is_some() && available.is_some() {
            break;
        }
    }

    Some(MemoryInfo {
        total: total? * KB_TO_BYTES,
        available: available? * KB_TO_BYTES,
    })
}

fn parse_kb_value(s: &str) -> Option<u64> {
    let trimmed = s.trim();
    trimmed.strip_suffix("kB").unwrap_or(trimmed).trim().parse().ok()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;

    #[test]
    fn parse_meminfo_typical() {
        let input = "\
MemTotal:       32768000 kB
MemFree:         8000000 kB
MemAvailable:   16000000 kB
Buffers:          500000 kB
";
        let info = parse_meminfo(input).unwrap();
        assert_eq!(info.total, 32_768_000 * 1024);
        assert_eq!(info.available, 16_000_000 * 1024);
        assert_eq!(info.used(), (32_768_000 - 16_000_000) * 1024);
        assert_eq!(info.total_gb(), 31);
    }

    #[test]
    fn parse_meminfo_missing_available_returns_none() {
        let input = "MemTotal:       32768000 kB\n";
        assert!(parse_meminfo(input).is_none());
    }

    #[test]
    fn parse_meminfo_missing_total_returns_none() {
        let input = "MemAvailable:   16000000 kB\n";
        assert!(parse_meminfo(input).is_none());
    }

    #[test]
    fn parse_kb_value_handles_whitespace() {
        assert_eq!(parse_kb_value("  12345 kB"), Some(12345));
        assert_eq!(parse_kb_value("  12345  kB"), Some(12345));
    }

    #[test]
    fn parse_kb_value_rejects_garbage() {
        assert!(parse_kb_value("not_a_number kB").is_none());
    }

    #[test]
    fn memory_info_percentages() {
        let info = MemoryInfo {
            total: 16 * 1024 * 1024 * 1024,
            available: 8 * 1024 * 1024 * 1024,
        };
        let pct = info.usage_percent();
        assert!((pct - 50.0).abs() < 0.01);
        assert!((info.available_gb() - 8.0).abs() < 0.01);
    }

    #[test]
    fn memory_info_zero_total() {
        let info = MemoryInfo {
            total: 0,
            available: 0,
        };
        assert!((info.usage_percent() - 0.0).abs() < f64::EPSILON);
        assert_eq!(info.used(), 0);
    }

    #[test]
    fn disk_info_total_gb_conversion() {
        let d = DiskInfo {
            device: "sda".to_string(),
            total_bytes: 500 * 1024 * 1024 * 1024,
        };
        assert!((d.total_gb() - 500.0).abs() < 0.01);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn memory_info_reads_proc() {
        let info = memory_info().expect("/proc/meminfo should be readable on Linux");
        assert!(info.total > 0, "total memory must be positive");
        assert!(info.available > 0, "available memory must be positive");
        assert!(info.available <= info.total);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn total_memory_gb_is_reasonable() {
        let gb = total_memory_gb();
        assert!(gb >= 1, "at least 1 GB expected, got {gb}");
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn disk_info_finds_at_least_one_disk() {
        let disks = disk_info();
        assert!(!disks.is_empty(), "should find at least one real block device");
        for disk in &disks {
            assert!(disk.total_bytes > 0, "disk {} should have nonzero size", disk.device);
        }
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn total_disk_gb_is_reasonable() {
        let gb = total_disk_gb().expect("should detect disk on Linux");
        assert!(gb >= 1, "at least 1 GB expected, got {gb}");
    }
}
