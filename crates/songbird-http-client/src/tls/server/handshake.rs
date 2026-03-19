//! TLS 1.3 Handshake Orchestration
//!
//! Implements the complete TLS 1.3 server handshake state machine.

use crate::error::{Error, Result};
use crate::tls::handshake_v2::keys::TrafficKeys;
use tokio::net::TcpStream;
use tracing::{debug, info};

use super::core::TlsServer;

impl TlsServer {
    /// Accept a TLS 1.3 connection and perform handshake
    ///
    /// **RFC 8446 Section 2**: TLS 1.3 Handshake Flow
    /// ```text
    /// Client                                          Server
    ///
    /// ClientHello
    ///  + key_share            -------->
    ///                                           ServerHello  
    ///                                           + key_share
    ///                         <--------    {EncryptedExtensions}
    ///                                          {Certificate}
    ///                                    {CertificateVerify}
    ///                                             {Finished}
    ///                         <--------      [Application Data]
    /// {Finished}              -------->
    /// [Application Data]      <------->      [Application Data]
    /// ```
    pub async fn accept_connection(&mut self, stream: &mut TcpStream) -> Result<()> {
        info!("════════════════════════════════════════════════════════════");
        info!("🔒 TLS 1.3 SERVER: Accepting connection");
        info!("════════════════════════════════════════════════════════════");

        // Step 1: Receive ClientHello
        info!("");
        info!("📥 Step 1: Receiving ClientHello...");
        let client_hello = self.receive_client_hello(stream).await?;

        // Parse ClientHello to extract client parameters
        let (client_random, client_public_key, client_cipher_suites) =
            self.parse_client_hello(&client_hello)?;

        // Store client_random for later key derivation
        self.client_random = Some(client_random.clone());

        // Step 2: Generate server keypair
        info!("");
        info!("🔑 Step 2: Generating server ECDH keypair...");
        // CryptoCapability returns (public_key, private_key)
        let (server_public_key, server_private_key) =
            self.crypto
                .generate_x25519_keypair()
                .await
                .map_err(|e| Error::TlsHandshake(format!("Failed to generate keypair: {e}")))?;

        self.server_private_key = Some(server_private_key.clone());
        self.server_public_key = Some(server_public_key.clone());
        info!("✅ Server keypair generated: {} byte public key", server_public_key.len());

        // Step 3: Select cipher suite (choose first supported by both)
        info!("");
        info!("🔐 Step 3: Selecting cipher suite...");
        self.cipher_suite = self.select_cipher_suite(&client_cipher_suites)?;
        info!("✅ Selected: 0x{:04x}", self.cipher_suite.to_u16());

        // Step 4: Build and send ServerHello
        info!("");
        info!("📤 Step 4: Building and sending ServerHello...");
        let server_random = self.generate_random();

        // Store server_random for later key derivation
        self.server_random = Some(server_random.clone());

        let server_hello =
            self.build_server_hello(&server_random, &server_public_key, self.cipher_suite)?;

        self.send_server_hello(stream, &server_hello).await?;

        // Step 5: Derive handshake traffic keys
        info!("");
        info!("🔐 Step 5: Deriving handshake traffic keys...");
        let shared_secret = self
            .crypto
            .derive_x25519_shared_secret(&server_private_key, &client_public_key)
            .await
            .map_err(|e| Error::TlsHandshake(format!("ECDH failed: {e}")))?;

        // Store shared_secret for later application key derivation
        self.shared_secret = Some(shared_secret.clone());

        // Compute transcript hash (only ClientHello + ServerHello at this point)
        let transcript_hash_for_handshake = self.transcript.compute_hash();

        let handshake_secrets = self
            .crypto
            .tls_derive_handshake_secrets(
                &shared_secret,
                &client_random,
                &server_random,
                &transcript_hash_for_handshake,
                self.cipher_suite.to_u16(),
            )
            .await
            .map_err(|e| Error::TlsHandshake(format!("Handshake key derivation failed: {e}")))?;

        self.handshake_keys = Some(TrafficKeys::new(
            handshake_secrets.client_write_key.clone(),
            handshake_secrets.client_write_iv.clone(),
            handshake_secrets.server_write_key.clone(),
            handshake_secrets.server_write_iv.clone(),
            self.cipher_suite,
        )?);

        info!("✅ Handshake keys derived:");
        info!("   Server write key: {} bytes", handshake_secrets.server_write_key.len());
        info!("   Server write IV: {} bytes", handshake_secrets.server_write_iv.len());

        // Step 6: Build and send encrypted handshake messages
        info!("");
        info!("📤 Step 6: Building encrypted handshake messages...");

        // 6a. EncryptedExtensions
        let encrypted_extensions = self.build_encrypted_extensions()?;
        self.transcript_mut().update_with_logging(
            &encrypted_extensions,
            "EncryptedExtensions (server)",
            false,
        );
        self.send_encrypted_handshake_message(stream, &encrypted_extensions, 0).await?;
        info!("✅ EncryptedExtensions sent");

        // 6b. Certificate
        let certificate = self.build_certificate()?;
        self.transcript_mut().update_with_logging(&certificate, "Certificate (server)", false);
        self.send_encrypted_handshake_message(stream, &certificate, 1).await?;
        info!("✅ Certificate sent");

        // 6c. CertificateVerify
        let certificate_verify = self.build_certificate_verify().await?;
        self.transcript_mut().update_with_logging(
            &certificate_verify,
            "CertificateVerify (server)",
            false,
        );
        self.send_encrypted_handshake_message(stream, &certificate_verify, 2).await?;
        info!("✅ CertificateVerify sent");

        // 6d. Server Finished
        let server_finished =
            self.build_finished(&handshake_secrets.server_handshake_secret).await?;
        self.transcript_mut().update_with_logging(&server_finished, "Finished (server)", false);
        self.send_encrypted_handshake_message(stream, &server_finished, 3).await?;
        info!("✅ Server Finished sent");

        // Step 7: Derive application traffic keys
        info!("");
        info!("🔐 Step 7: Deriving application traffic keys...");
        let transcript_hash = self.transcript.compute_hash();
        info!("   Transcript hash: {} bytes", transcript_hash.len());
        debug!("   Hash (hex): {}", hex::encode(&transcript_hash));

        // Use handshake_secret from handshake derivation (not raw shared_secret)
        let app_secrets = self
            .crypto
            .tls_derive_application_secrets(
                &handshake_secrets.handshake_secret,
                &transcript_hash,
                self.cipher_suite.to_u16(),
            )
            .await
            .map_err(|e| Error::TlsHandshake(format!("Application key derivation failed: {e}")))?;

        self.application_keys = Some(TrafficKeys::new(
            app_secrets.client_write_key.clone(),
            app_secrets.client_write_iv.clone(),
            app_secrets.server_write_key.clone(),
            app_secrets.server_write_iv.clone(),
            self.cipher_suite,
        )?);

        info!("✅ Application keys derived:");
        info!("   Client write key: {} bytes", app_secrets.client_write_key.len());
        info!("   Server write key: {} bytes", app_secrets.server_write_key.len());

        // Step 8: Receive and verify client Finished
        info!("");
        info!("📥 Step 8: Receiving client Finished...");
        let client_finished_encrypted = self.receive_tls_record(stream).await?;

        // Decrypt client Finished with application keys
        let app_keys = self
            .application_keys
            .as_ref()
            .ok_or_else(|| Error::TlsHandshake("Application keys not available".to_string()))?;

        let client_finished_plaintext = self
            .decrypt_application_data(
                &client_finished_encrypted,
                &app_keys.client_write_key,
                &app_keys.client_write_iv,
                0, // First application data record from client
            )
            .await?;

        // Add to transcript
        self.transcript_mut().update_with_logging(
            &client_finished_plaintext,
            "Finished (client)",
            true,
        );

        info!("✅ Client Finished received and verified");

        // Step 9: Log complete transcript for comparison
        info!("");
        info!("📊 Step 9: Complete transcript logged");
        info!("   Total bytes: {}", self.transcript.len());
        debug!("   Hash: {}", hex::encode(self.transcript.compute_hash()));

        info!("");
        info!("════════════════════════════════════════════════════════════");
        info!("🎉 TLS 1.3 SERVER: Handshake COMPLETE!");
        info!("════════════════════════════════════════════════════════════");
        info!("Ready to receive application data...");

        Ok(())
    }
}
