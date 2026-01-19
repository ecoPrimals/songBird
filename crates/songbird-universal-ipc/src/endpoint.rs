//! Endpoint types - Virtual and Native

use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// Virtual endpoint (platform-agnostic path)
///
/// Always uses Unix-style paths like `/primal/beardog`.
/// The universal IPC layer translates these to platform-specific endpoints.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VirtualEndpoint {
    /// Virtual path (always Unix-style)
    pub path: String,
}

impl VirtualEndpoint {
    /// Create a new virtual endpoint
    ///
    /// # Arguments
    /// * `primal_name` - Name of the primal (e.g., "beardog")
    ///
    /// # Returns
    /// Virtual endpoint with path `/primal/{name}`
    pub fn new(primal_name: &str) -> Self {
        Self {
            path: format!("/primal/{}", primal_name),
        }
    }

    /// Extract primal name from virtual path
    ///
    /// # Returns
    /// Primal name, or None if path format is invalid
    pub fn primal_name(&self) -> Option<&str> {
        self.path.strip_prefix("/primal/")
    }

    /// Validate virtual path format
    pub fn is_valid(&self) -> bool {
        self.primal_name().is_some()
    }
}

/// Native endpoint (platform-specific)
///
/// This is the actual endpoint used by the OS:
/// - Unix: `/tmp/primal-beardog.sock`
/// - Windows: `\\.\pipe\primal-beardog`
/// - Fallback: `127.0.0.1:{port}`
#[derive(Debug, Clone)]
pub enum NativeEndpoint {
    /// Unix domain socket (Linux, macOS, BSD, etc.)
    #[cfg(unix)]
    UnixSocket(PathBuf),

    /// Windows named pipe
    #[cfg(windows)]
    NamedPipe(String),

    /// TCP localhost (fallback for platforms without Unix sockets or named pipes)
    TcpLocal(u16),
}

impl NativeEndpoint {
    /// Get display string for logging
    pub fn display(&self) -> String {
        match self {
            #[cfg(unix)]
            NativeEndpoint::UnixSocket(path) => {
                format!("unix://{}", path.display())
            }
            #[cfg(windows)]
            NativeEndpoint::NamedPipe(name) => {
                format!("pipe://{}", name)
            }
            NativeEndpoint::TcpLocal(port) => {
                format!("tcp://127.0.0.1:{}", port)
            }
        }
    }

    /// Check if endpoint exists/is accessible
    #[cfg(unix)]
    pub fn exists(&self) -> bool {
        match self {
            NativeEndpoint::UnixSocket(path) => path.exists(),
            NativeEndpoint::TcpLocal(_) => true, // TCP always "exists"
        }
    }

    #[cfg(windows)]
    pub fn exists(&self) -> bool {
        // Named pipes don't have a simple "exists" check
        // We'd need to try opening, which is expensive
        // For now, assume exists
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_virtual_endpoint_creation() {
        let endpoint = VirtualEndpoint::new("beardog");
        assert_eq!(endpoint.path, "/primal/beardog");
        assert_eq!(endpoint.primal_name(), Some("beardog"));
        assert!(endpoint.is_valid());
    }

    #[test]
    fn test_virtual_endpoint_invalid() {
        let endpoint = VirtualEndpoint {
            path: "/invalid/path".to_string(),
        };
        assert_eq!(endpoint.primal_name(), None);
        assert!(!endpoint.is_valid());
    }

    #[test]
    #[cfg(unix)]
    fn test_native_endpoint_display_unix() {
        let endpoint = NativeEndpoint::UnixSocket(PathBuf::from("/tmp/test.sock"));
        assert_eq!(endpoint.display(), "unix:///tmp/test.sock");
    }

    #[test]
    fn test_native_endpoint_display_tcp() {
        let endpoint = NativeEndpoint::TcpLocal(8080);
        assert_eq!(endpoint.display(), "tcp://127.0.0.1:8080");
    }
}

