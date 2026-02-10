//! Platform-specific NFC device abstraction
//!
//! Provides unified interface across Android, iOS, Linux, etc.

use crate::error::{NfcError, Result};
use crate::protocol::NfcMessage;
use std::time::Duration;
use tracing::{debug, info};

/// NFC device abstraction
///
/// Platform-agnostic interface for NFC operations
pub struct NfcDevice {
    /// Platform-specific backend
    backend: Box<dyn NfcBackend>,

    /// Connection timeout
    timeout: Duration,
}

impl std::fmt::Debug for NfcDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NfcDevice").field("timeout", &self.timeout).finish_non_exhaustive()
    }
}

impl NfcDevice {
    /// Create new NFC device
    pub fn new(timeout: Duration) -> Result<Self> {
        let backend = Self::create_platform_backend()?;

        Ok(Self {
            backend,
            timeout,
        })
    }

    /// Connect to peer device
    pub async fn connect(&mut self) -> Result<()> {
        info!("Connecting to NFC peer");
        self.backend.connect(self.timeout).await
    }

    /// Disconnect from peer
    pub async fn disconnect(&mut self) -> Result<()> {
        info!("Disconnecting from NFC peer");
        self.backend.disconnect().await
    }

    /// Send raw bytes
    pub async fn send_raw(&mut self, data: &[u8]) -> Result<()> {
        debug!("Sending {} bytes", data.len());
        self.backend.send(data).await
    }

    /// Receive raw bytes
    pub async fn receive_raw(&mut self, expected_len: usize) -> Result<Vec<u8>> {
        debug!("Receiving {} bytes", expected_len);
        self.backend.receive(expected_len).await
    }

    /// Send NFC message
    pub async fn send_message(&mut self, message: &NfcMessage) -> Result<()> {
        let bytes = message.to_bytes()?;
        self.send_raw(&bytes).await
    }

    /// Receive NFC message
    pub async fn receive_message(&mut self) -> Result<NfcMessage> {
        // First receive header to get payload length
        let header = self.receive_raw(crate::HEADER_SIZE).await?;

        let payload_len = u16::from_be_bytes([header[2], header[3]]) as usize;
        let total_len = crate::FRAME_OVERHEAD + payload_len;

        // Receive full frame
        let mut full_frame = header;
        let remaining = self.receive_raw(total_len - crate::HEADER_SIZE).await?;
        full_frame.extend_from_slice(&remaining);

        NfcMessage::from_bytes(&full_frame)
    }

    /// Create platform-specific backend
    fn create_platform_backend() -> Result<Box<dyn NfcBackend>> {
        #[cfg(target_os = "android")]
        {
            Ok(Box::new(AndroidNfcBackend::new()?))
        }

        #[cfg(target_os = "ios")]
        {
            Ok(Box::new(IosNfcBackend::new()?))
        }

        #[cfg(target_os = "linux")]
        {
            Ok(Box::new(LinuxNfcBackend::new()?))
        }

        #[cfg(not(any(target_os = "android", target_os = "ios", target_os = "linux")))]
        {
            Err(NfcError::Platform("Unsupported platform".to_string()))
        }
    }
}

/// Platform-specific NFC backend trait
#[async_trait::async_trait]
pub trait NfcBackend: Send + Sync {
    /// Connect to peer
    async fn connect(&mut self, timeout: Duration) -> Result<()>;

    /// Disconnect from peer
    async fn disconnect(&mut self) -> Result<()>;

    /// Send data
    async fn send(&mut self, data: &[u8]) -> Result<()>;

    /// Receive data
    async fn receive(&mut self, expected_len: usize) -> Result<Vec<u8>>;
}

// ========== Platform Implementations (Stubs - TODO: Real platform integration) ==========

#[cfg(target_os = "android")]
struct AndroidNfcBackend;

#[cfg(target_os = "android")]
impl AndroidNfcBackend {
    fn new() -> Result<Self> {
        // TODO: Initialize Android NFC via JNI
        Ok(Self)
    }
}

#[cfg(target_os = "android")]
#[async_trait::async_trait]
impl NfcBackend for AndroidNfcBackend {
    async fn connect(&mut self, _timeout: Duration) -> Result<()> {
        // TODO: Android NFC connection via JNI
        Err(NfcError::Platform("Android NFC not yet implemented".to_string()))
    }

    async fn disconnect(&mut self) -> Result<()> {
        Ok(())
    }

    async fn send(&mut self, _data: &[u8]) -> Result<()> {
        Err(NfcError::Platform("Android NFC not yet implemented".to_string()))
    }

    async fn receive(&mut self, _expected_len: usize) -> Result<Vec<u8>> {
        Err(NfcError::Platform("Android NFC not yet implemented".to_string()))
    }
}

#[cfg(target_os = "ios")]
struct IosNfcBackend;

#[cfg(target_os = "ios")]
impl IosNfcBackend {
    fn new() -> Result<Self> {
        // TODO: Initialize iOS CoreNFC
        Ok(Self)
    }
}

#[cfg(target_os = "ios")]
#[async_trait::async_trait]
impl NfcBackend for IosNfcBackend {
    async fn connect(&mut self, _timeout: Duration) -> Result<()> {
        // TODO: iOS CoreNFC connection
        Err(NfcError::Platform("iOS NFC not yet implemented".to_string()))
    }

    async fn disconnect(&mut self) -> Result<()> {
        Ok(())
    }

    async fn send(&mut self, _data: &[u8]) -> Result<()> {
        Err(NfcError::Platform("iOS NFC not yet implemented".to_string()))
    }

    async fn receive(&mut self, _expected_len: usize) -> Result<Vec<u8>> {
        Err(NfcError::Platform("iOS NFC not yet implemented".to_string()))
    }
}

#[cfg(target_os = "linux")]
struct LinuxNfcBackend;

#[cfg(target_os = "linux")]
impl LinuxNfcBackend {
    #[allow(clippy::unnecessary_wraps)] // Result kept for consistency with other platform backends
    fn new() -> Result<Self> {
        // TODO: Initialize libnfc
        Ok(Self)
    }
}

#[cfg(target_os = "linux")]
#[async_trait::async_trait]
impl NfcBackend for LinuxNfcBackend {
    async fn connect(&mut self, _timeout: Duration) -> Result<()> {
        // TODO: libnfc connection
        Err(NfcError::Platform("Linux NFC not yet implemented".to_string()))
    }

    async fn disconnect(&mut self) -> Result<()> {
        Ok(())
    }

    async fn send(&mut self, _data: &[u8]) -> Result<()> {
        Err(NfcError::Platform("Linux NFC not yet implemented".to_string()))
    }

    async fn receive(&mut self, _expected_len: usize) -> Result<Vec<u8>> {
        Err(NfcError::Platform("Linux NFC not yet implemented".to_string()))
    }
}
