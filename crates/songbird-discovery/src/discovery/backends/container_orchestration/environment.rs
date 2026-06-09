// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Container environment heuristics (env vars, filesystem, PID 1) and discovery from those signals.

use super::types::{OrchestrationMethod, UniversalContainerOrchestration};
use crate::traits::ServiceQuery;
use crate::traits::service::ServiceInfo;
use songbird_types::errors::SongbirdResult;
use std::collections::HashMap;
use tracing::debug;

impl UniversalContainerOrchestration {
    /// Detect container environment
    pub(super) fn detect_container_environment(&mut self) {
        // Check for various container environment indicators
        let container_indicators = vec![
            ("KUBERNETES_SERVICE_HOST", "kubernetes"),
            ("DOCKER_HOST", "docker"),
            ("CONTAINER", "generic"),
        ];

        for (env_var, runtime) in container_indicators {
            if songbird_process_env::var(env_var).is_ok() {
                debug!("Detected container environment: {} ({})", runtime, env_var);
                if self
                    .orchestration_methods
                    .iter()
                    .all(|m| !matches!(m, OrchestrationMethod::ContainerEnvironment))
                {
                    self.orchestration_methods.push(OrchestrationMethod::ContainerEnvironment);
                }

                if self.runtime_info.runtime_type == "unknown" {
                    self.runtime_info.runtime_type = runtime.to_string();
                }
                break;
            }
        }

        // Check for container filesystem indicators
        let container_files = vec!["/.dockerenv", "/run/.containerenv", "/proc/1/cgroup"];

        for file_path in container_files {
            if std::path::Path::new(file_path).exists() {
                debug!("Detected container environment via filesystem: {}", file_path);
                if self
                    .orchestration_methods
                    .iter()
                    .all(|m| !matches!(m, OrchestrationMethod::ContainerEnvironment))
                {
                    self.orchestration_methods.push(OrchestrationMethod::ContainerEnvironment);
                }

                if self.runtime_info.runtime_type == "unknown" {
                    self.runtime_info.runtime_type = "container".to_string();
                }
                break;
            }
        }
    }

    /// Detect process-based containers
    pub(super) fn detect_process_based_containers(&mut self) {
        // Check if running as PID 1 (common in containers)
        if std::process::id() == 1 {
            debug!("Detected container environment: running as PID 1");
            self.orchestration_methods.push(OrchestrationMethod::ProcessBased);

            if self.runtime_info.runtime_type == "unknown" {
                self.runtime_info.runtime_type = "container".to_string();
            }
        }
    }

    /// Discover services from container environment
    pub(super) async fn discover_from_container_environment(
        &self,
        _query: &ServiceQuery,
    ) -> SongbirdResult<Vec<ServiceInfo>> {
        let mut services = Vec::new();
        let env_vars = songbird_process_env::vars().collect::<HashMap<_, _>>();

        // Look for service-related environment variables common in containers
        for (key, _value) in env_vars {
            if key.ends_with("_SERVICE_HOST") || key.ends_with("_PORT") {
                let service_name =
                    key.replace("_SERVICE_HOST", "").replace("_PORT", "").to_lowercase();
                if !service_name.is_empty() {
                    services.push(self.create_service_info(&service_name, "container-env"));
                }
            }
        }

        Ok(services)
    }

    /// Discover services by scanning /proc for processes with listening TCP sockets.
    ///
    /// Reads `/proc/net/tcp` for listening sockets, then correlates with `/proc/<pid>/fd`
    /// symlinks to identify which processes own them.
    pub(super) async fn discover_from_process_based(
        &self,
        query: &ServiceQuery,
    ) -> SongbirdResult<Vec<ServiceInfo>> {
        let mut services = Vec::new();

        let Ok(tcp_contents) = std::fs::read_to_string("/proc/net/tcp") else {
            debug!("Cannot read /proc/net/tcp — process-based discovery unavailable");
            return Ok(services);
        };
        let listening_inodes = parse_listening_inodes(&tcp_contents);

        if listening_inodes.is_empty() {
            return Ok(services);
        }

        let proc_dir = match std::fs::read_dir("/proc") {
            Ok(d) => d,
            Err(_) => return Ok(services),
        };

        for entry in proc_dir.flatten() {
            let pid_str = entry.file_name();
            let pid_str = pid_str.to_string_lossy();
            if !pid_str.chars().all(|c| c.is_ascii_digit()) {
                continue;
            }

            let fd_dir = entry.path().join("fd");
            let fds = match std::fs::read_dir(&fd_dir) {
                Ok(d) => d,
                Err(_) => continue,
            };

            let mut owns_listener = false;
            for fd_entry in fds.flatten() {
                if let Ok(link) = std::fs::read_link(fd_entry.path()) {
                    let link_str = link.to_string_lossy();
                    if let Some(inode_str) =
                        link_str.strip_prefix("socket:[").and_then(|s| s.strip_suffix(']'))
                    {
                        if let Ok(inode) = inode_str.parse::<u64>() {
                            if listening_inodes.contains(&inode) {
                                owns_listener = true;
                                break;
                            }
                        }
                    }
                }
            }

            if !owns_listener {
                continue;
            }

            let comm_path = entry.path().join("comm");
            let name = std::fs::read_to_string(&comm_path).unwrap_or_default().trim().to_string();

            if name.is_empty() || name == "songbird" {
                continue;
            }

            if let Some(ref name_filter) = query.name {
                if !name.contains(name_filter.as_str()) {
                    continue;
                }
            }

            services.push(self.create_service_info(&name, "process"));
        }

        debug!("Process-based discovery found {} services", services.len());
        Ok(services)
    }
}

/// Parse /proc/net/tcp for inodes of sockets in LISTEN state (0A).
fn parse_listening_inodes(contents: &str) -> Vec<u64> {
    contents
        .lines()
        .skip(1)
        .filter_map(|line| {
            let fields: Vec<&str> = line.split_whitespace().collect();
            // field[3] = state (0A = LISTEN), field[9] = inode
            if fields.len() >= 10 && fields[3] == "0A" {
                fields[9].parse::<u64>().ok()
            } else {
                None
            }
        })
        .collect()
}
