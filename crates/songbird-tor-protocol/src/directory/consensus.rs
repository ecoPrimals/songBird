//! Tor network consensus
//!
//! Fetches and parses the Tor network consensus document from directory authorities.

use crate::crypto::BeardogCryptoClient;
use crate::directory::{DirectoryAuthority, RelayInfo, CircuitPath};
use crate::error::{Error, Result};
use crate::directory::authorities::DIRECTORY_AUTHORITIES;
use std::time::{Duration, SystemTime};
use tracing::{debug, info, warn};

/// Tor network consensus
#[derive(Debug, Clone)]
pub struct Consensus {
    /// When this consensus is valid from
    pub valid_after: SystemTime,
    /// When this consensus is fresh until
    pub fresh_until: SystemTime,
    /// When this consensus is valid until
    pub valid_until: SystemTime,
    /// List of relays in the network
    pub relays: Vec<RelayInfo>,
}

impl Consensus {
    /// Fetch consensus from directory authorities
    ///
    /// Tries multiple authorities until successful.
    pub async fn fetch(beardog: &BeardogCryptoClient) -> Result<Self> {
        info!("Fetching Tor network consensus");
        
        // Try up to 3 authorities
        for authority in DIRECTORY_AUTHORITIES.iter().take(3) {
            debug!("Trying directory authority: {}", authority.nickname);
            
            match Self::fetch_from_authority(authority, beardog).await {
                Ok(consensus) => {
                    info!(
                        authority = authority.nickname,
                        relays = consensus.relays.len(),
                        "Successfully fetched consensus"
                    );
                    return Ok(consensus);
                }
                Err(e) => {
                    warn!(
                        authority = authority.nickname,
                        error = %e,
                        "Failed to fetch from authority, trying next"
                    );
                    continue;
                }
            }
        }
        
        Err(Error::Consensus(
            "Failed to fetch consensus from all authorities".to_string()
        ))
    }
    
    /// Fetch consensus from specific authority
    async fn fetch_from_authority(
        authority: &DirectoryAuthority,
        _beardog: &BeardogCryptoClient,
    ) -> Result<Self> {
        let url = authority.consensus_url();
        
        debug!("Fetching consensus from: {}", url);
        
        // HTTP GET with timeout
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;
        
        let response = client.get(&url).send().await?;
        
        if !response.status().is_success() {
            return Err(Error::Http(reqwest::Error::from(
                response.error_for_status().expect_err("Status was not success")
            )));
        }
        
        let body = response.text().await?;
        
        debug!("Consensus downloaded, size: {} bytes", body.len());
        
        // Parse consensus
        Self::parse(&body)
    }
    
    /// Parse consensus document
    fn parse(_data: &str) -> Result<Self> {
        use crate::directory::parser::parse_consensus;
        
        debug!("Parsing consensus with nom");
        
        // Parse relay entries
        let relays = parse_consensus(_data)?;
        
        debug!("Parsed {} relays from consensus", relays.len());
        
        // Extract timestamps (look for valid-after, fresh-until, valid-until)
        // TODO: Parse timestamps properly
        let now = SystemTime::now();
        
        Ok(Self {
            valid_after: now,
            fresh_until: now + Duration::from_secs(3600),
            valid_until: now + Duration::from_secs(7200),
            relays,
        })
    }
    
    /// Select a circuit path (guard -> middle -> exit/hsdir)
    pub fn select_path(&self) -> Result<CircuitPath> {
        // TODO Phase 2A: Implement intelligent relay selection
        
        // For now, require at least 3 relays
        if self.relays.len() < 3 {
            return Err(Error::Consensus("Not enough relays in consensus".to_string()));
        }
        
        // Select guard (must have GUARD flag)
        let guard = self.relays.iter()
            .find(|r| r.is_guard())
            .ok_or_else(|| Error::Consensus("No suitable guard found".to_string()))?
            .clone();
        
        // Select middle (must be fast + stable)
        let middle = self.relays.iter()
            .find(|r| r.is_middle() && r.fingerprint != guard.fingerprint)
            .ok_or_else(|| Error::Consensus("No suitable middle found".to_string()))?
            .clone();
        
        // Select exit/hsdir (must have HSDIR flag for onion services)
        let exit = self.relays.iter()
            .find(|r| {
                r.is_hsdir()
                    && r.fingerprint != guard.fingerprint
                    && r.fingerprint != middle.fingerprint
            })
            .ok_or_else(|| Error::Consensus("No suitable hsdir found".to_string()))?
            .clone();
        
        Ok(CircuitPath { guard, middle, exit })
    }
    
    /// Check if consensus is still fresh
    pub fn is_fresh(&self) -> bool {
        SystemTime::now() < self.fresh_until
    }
    
    /// Check if consensus is still valid
    pub fn is_valid(&self) -> bool {
        SystemTime::now() < self.valid_until
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_empty() {
        // Empty string will fail since parser expects "r " prefix
        let result = Consensus::parse("");
        // Parser fails on empty input (expects at least "r " line)
        // This is expected behavior - empty consensus is invalid
        assert!(result.is_err());
    }
    
    #[test]
    fn test_is_fresh() {
        let consensus = Consensus {
            valid_after: SystemTime::now() - Duration::from_secs(3600),
            fresh_until: SystemTime::now() + Duration::from_secs(3600),
            valid_until: SystemTime::now() + Duration::from_secs(7200),
            relays: vec![],
        };
        
        assert!(consensus.is_fresh());
        assert!(consensus.is_valid());
    }
}
