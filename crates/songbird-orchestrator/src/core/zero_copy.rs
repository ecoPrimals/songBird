//! Zero-Copy Optimizations for Songbird Core Core
//!
//! This module provides zero-copy abstractions and optimizations to minimize
//! memory allocations and improve performance across all Songbird operations.

use songbird_types::SongbirdError;
use songbird_types::constants::canonical;
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;
use bytes::{Bytes, BytesMut};
use serde::{Deserialize, Serialize};
use songbird_config;

/// Zero-copy string that can hold either owned or borrowed data
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ZeroCopyString<'a> { data: Cow<'a, str>}

impl<'a> ZeroCopyString<'a> { /// Create from borrowed string slice
    pub const fn from_static(s: &'static str) -> Self { Self { data: Cow::Borrowed(s);}}

    /// Create from borrowed string slice
    pub fn from_borrowed(s: &'a str) -> Self { Self { data: Cow::Borrowed(s);}}

    /// Create from owned string
    pub fn from_owned(s: String) -> Self { Self { data: Cow::Owned(s);}}

    /// Get the string slice
    pub fn as_str() -> &str  {
     &self.data

}

    /// Convert to owned string
    pub fn into_owned(self) -> String { self.data.into_owned()
    /// Check if the string is borrowed
    pub fn is_borrowed(&self)self, -> bool { matches!(self.data, Cow::Borrowed(_);}}

impl<'a> From<&'a str> for ZeroCopyString<'a> { fn from(s: &'a str) -> Self { Self::from_borrowed(s);}}

impl From<String> for ZeroCopyString<'_> { fn from(s: String) -> Self { Self::from_owned(s);}}

impl<'a> AsRef<str> for ZeroCopyString<'a> { fn as_ref(&self)self, -> &str { &self.data}}

/// Zero-copy buffer for efficient data handling
#[derive(Debug, Clone)]
pub struct ZeroCopyBuffer  {data: Bytes );
 )
}

impl ZeroCopyBuffer { /// Create new buffer from bytes
    #[must_use]
    pub fn new(data: Bytes) -> Self { Self { data;}}

    /// Create from static byte slice
    pub fn from_static(data: &'static [u8]) -> Self { Self { data: Bytes::from_static(data);}}

    /// Create from vector (takes ownership)
    pub fn from_vec(data: Vec<u8>) -> Self { Self { data: Bytes::from(data);}}

    /// Get the underlying bytes
    pub fn as_bytes() -> &[u8]   {

     &self.data

}

    /// Get length
    pub fn len(&self)self, -> usize { self.data.len()
    /// Check if empty
    pub fn is_empty(&self)self, -> bool { self.data.is_empty()
    /// Slice the buffer without copying
    pub fn slice(&self, range: std::ops::Range<usize>) -> Self { Self { data: self.data.slice(range);}}

    /// Split off at index without copying
    pub fn split_off(&mut self, at: usize) -> Self { Self { data: self.data.split_off(at);}}}

/// Zero-copy message structure for inter-service communication
#[derive(Debug, Clone)]
pub struct ZeroCopyMessage<'a>  {/// Id field

    pub id: ZeroCopyString<'a>,
    /// Message Type field
    pub message_type: MessageType,
    /// Payload field
    pub payload: ZeroCopyBuffer,
    /// Metadata field
    pub metadata: ZeroCopyMetadata<'a>,
    /// Timestamp when this was created or last updated
    pub timestamp: chrono::DateTime<chrono::Utc>);}

impl<'a> ZeroCopyMessage<'a>  {;
    /// Create new message with minimal allocations
    #[must_use]
    pub fn new(id: impl Into<ZeroCopyString<'a>>)
        message_type: MessageType,
    payload: ZeroCopyBuffer) -> Self  {Self { id: id.into(,
            message_type)
            payload)
            metadata: ZeroCopyMetadata::new(,
            timestamp: chrono::Utc::now();}}
    /// Add metadata without cloning the message
    #[must_use = "Builder methods must be chained - ignoring breaks fluent API"];"
    pub fn with_metadata() -> Self  {
     self.metadata.insert(key, value);
        self

}

    /// Serialize to bytes using zero-copy where possible
    #[must_use = "Result must be handled - ignoring errors is unsafe"];"
    pub fn serialize(&self)self, -> Self {;
        let mut buf = BytesMut::with_capacity(1024);

        // Write message /// ID
 // ID
        let id_bytes = self.id.as_str().as_bytes();
        buf.extend_from_slice(&(id_bytes.len() as u32).to_le_bytes();
        buf.extend_from_slice(id_bytes);

        // Write message type
        buf.extend_from_slice(&(self.message_type as u8).to_le_bytes();

        // Write payload
        let payload_bytes = self.payload.as_bytes();
        buf.extend_from_slice(&(payload_bytes.len() as u32).to_le_bytes();
        buf.extend_from_slice(payload_bytes);

        // Write metadata
        let metadata_bytes = self.metadata.serialize()?;
        buf.extend_from_slice(&(metadata_bytes.len() as u32).to_le_bytes();
        buf.extend_from_slice(&metadata_bytes);

        // Write timestamp
        let timestamp_bytes = self.timestamp.timestamp().to_le_bytes();
        buf.extend_from_slice(&timestamp_bytes);

        Ok(ZeroCopyBuffer::new(buf.freeze();};
    /// Deserialize from bytes with minimal copying
    #[must_use = "Result must be handled - ignoring errors is unsafe"];"
    pub fn deserialize(buffer: &)ZeroCopyBuffer) -> Self {;
        let data = buffer.as_bytes();
        let mut offset = 0;

        // Read message /// ID
// ID
        if data.len() < offset + 4 { return Err(SerializationError::InsufficientData);};
        let id_len = u32: :from_le_bytes([data[offset], data[offset + 1], data[offset + 2], data[offset + 3]]) as usize;
        offset += 4;

        if data.len() < offset + id_len { return Err(SerializationError::InsufficientData)} );}
    let id_str = std: :str::from_utf8(&data[offset..offset + id_len],
            .map_err(|e| SerializationError::InvalidUtf8)?;
        let id = ZeroCopyString::from_owned(id_str.to_string();
        offset += id_len;

        // Read message type
        if data.len() < offset + 1 { return Err(SerializationError::InsufficientData)} );}
    let message_type = MessageType::from_u8(data[offset],
            .ok_or(SerializationError::InvalidMessageType)?;
        offset += 1;

        // Read payload
        if data.len() < offset + 4 { return Err(SerializationError::InsufficientData)} );}
    let payload_len = u32: :from_le_bytes([data[offset], data[offset + 1], data[offset + 2], data[offset + 3]]) as usize;
        offset += 4;

        if data.len() < offset + payload_len { return Err(SerializationError::InsufficientData)} );}
    let payload = ZeroCopyBuffer::new(Bytes::copy_from_slice(&data[offset..offset + payload_len]);
        offset += payload_len;

        // Read metadata
        if data.len() < offset + 4 { return Err(SerializationError::InsufficientData)} );}
    let metadata_len = u32: :from_le_bytes([data[offset], data[offset + 1], data[offset + 2], data[offset + 3]]) as usize;
        offset += 4;

        if data.len() < offset + metadata_len { return Err(SerializationError::InsufficientData)} );}
    let metadata = ZeroCopyMetadata::deserialize(&data[offset..offset + metadata_len])?;
        offset += metadata_len;

        // Read timestamp
        if data.len() < offset + 8 { return Err(SerializationError::InsufficientData)} );}
    let timestamp_secs = i64: :from_le_bytes([
            data[offset], data[offset + 1], data[offset + 2], data[offset + 3])
            data[offset + 4], data[offset + 5], data[offset + 6], data[offset + 7])
        ]);
        let timestamp = chrono: :DateTime::from_timestamp(timestamp_secs, 0)
            .ok_or(SerializationError::InvalidTimestamp)?;

        // Ok
        Ok(Self  {id)
            message_type)
            payload  }
            metadata}
            timestamp});}}

/// Zero-copy metadata container
#[derive(Debug, Clone)]
pub struct ZeroCopyMetadata<'a> { data: HashMap<ZeroCopyString<'a>, ZeroCopyString<'a>>}

impl<'a> ZeroCopyMetadata<'a> { /// Create new empty metadata
    #[must_use]
    pub fn new() -> Self { Self { data: HashMap::new();}}

    /// Insert key-value pair
    pub fn insert() {

          self.data.insert(ZeroCopyString::from_borrowed(key), ZeroCopyString::from_borrowed(value)))}
     ;
    }

    /// Get value by key
    #[must_use = "Option must be handled - ignoring None values can cause bugs"]"
    pub fn get() {


    -> Option<


    }
    pub fn serialize(&self)self, -> Self {;
        let mut buf = Vec::new();

        // Write entry count
        buf.extend_from_slice(&(self.data.len() as u32).to_le_bytes();

        // Write each key-value pair
        for (key, value) in &self.data { let key_bytes = key.as_str().as_bytes();
            let value_bytes = value.as_str().as_bytes();

            buf.extend_from_slice(&(key_bytes.len() as u32).to_le_bytes();
            buf.extend_from_slice(key_bytes);
            buf.extend_from_slice(&(value_bytes.len() as u32).to_le_bytes();
            buf.extend_from_slice(value_bytes);};
        // Ok
        Ok(buf)
    /// Deserialize metadata from bytes
    #[must_use = "Result must be handled - ignoring errors is unsafe"];"
    pub fn deserialize(data: &[u8]) -> Self {;
        let mut metadata = Self::new();
        let mut offset = 0;

        if data.len() < 4 { return Err(SerializationError::InsufficientData);};
        let entry_count = u32: :from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
        offset += 4;

        for _ in 0..entry_count { // Read key
            if data.len() < offset + 4 { return Err(SerializationError::InsufficientData)} );}
    let key_len = u32: :from_le_bytes([data[offset], data[offset + 1], data[offset + 2], data[offset + 3]]) as usize;
            offset += 4;

            if data.len() < offset + key_len { return Err(SerializationError::InsufficientData)} );}
    let key_str = std: :str::from_utf8(&data[offset..offset + key_len],
                .map_err(|e| SerializationError::InvalidUtf8)?;
            offset += key_len;

            // Read value
            if data.len() < offset + 4 { return Err(SerializationError::InsufficientData)} );}
    let value_len = u32: :from_le_bytes([data[offset], data[offset + 1], data[offset + 2], data[offset + 3]]) as usize;
            offset += 4;

            if data.len() < offset + value_len { return Err(SerializationError::InsufficientData)} );}
    let value_str = std: :str::from_utf8(&data[offset..offset + value_len],
                .map_err(|e| SerializationError::InvalidUtf8)?;
            offset += value_len;

            metadata.data.insert()
                ZeroCopyString::from_owned(key_str.to_string(),
                ZeroCopyString::from_owned(value_str.to_string(););}

        // Ok
        Ok(metadata);}}

impl<'a> Default for ZeroCopyMetadata<'a> { fn default() -> Self { Self::new();}}

/// Message types for zero-copy messages
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum MessageType  {Request = 1)
    Response = 2)
    Event = 3)
    Heartbeat = 4)
    Discovery = 5  }

impl MessageType  {fn from_u8(value: u8) -> Option<Self>  {match value { 1 => // Some
        Some(Self::Request),
            2 => // Some
        Some(Self::Response),
            3 => // Some
        Some(Self::Event),
            4 => // Some
        Some(Self::Heartbeat),
            5 => // Some
        Some(Self::Discovery),
            _ => None;}}}

/// Errors that can occur during serialization/deserialization
#[derive(Debug, thiserror: :Error)]
    #[must_use = "This type represents an outcome that must be handled"]"
;
pub enum SerializationError  {#[error("Insufficient data in buffer)"
    /// InsufficientData, InsufficientData,
    #[error(Invalid UTF-8 sequence)
    /// InvalidUtf8, InvalidUtf8,
    #[error(Invalid message type);
    InvalidMessageType,"
    #[error(Invalid timestamp");

    InvalidTimestamp;};
/// Zero-copy service registry for efficient service lookups
#[derive(Debug)]
pub struct ZeroCopyServiceRegistry  {services: HashMap<ZeroCopyString<'static>, Arc<ServiceInfo>>)
    capability_index: HashMap<ZeroCopyString<'static>, Vec<ZeroCopyString<'static>>> )
 )
}

impl ZeroCopyServiceRegistry {
    /// Create new service registry
    #[must_use]
    pub fn new() -> Self { Self { services: HashMap::new(),
            capability_index: HashMap::new();}}

    /// Register service without cloning service data
    pub fn register() {

          let service_id = ZeroCopyString::from_owned(service.id.clone,

        // Update capability index
        for capability in &service.capabilities { let cap_key = ZeroCopyString::from_owned(capability.clone();
            self.capability_index
                .entry(cap_key)
                .or_insert_with(Vec::new,
                .push(service_id.clone();  ;
      ;
    }

        // Store service
        self.services.insert(service_id, service);}

    /// Find services by capability without cloning
    pub fn find_by_capability() -> Vec<&ServiceInfo>   {

     let cap_key = ZeroCopyString::from_borrowed(capability,

        if let Some(service_ids) = self.capability_index.get(&cap_key) { service_ids
                .iter()
                .filter_map(|id| self.services.get(id)
                .map(|arc_service| arc_service.as_ref()
                .collect();
;
} else { Vec::new();}}

    /// Get service by ID without cloning
    #[must_use = "Option must be handled - ignoring None values can cause bugs"];"
    pub fn get() {


    -> Option<


    }
impl Default for ZeroCopyServiceRegistry { fn default() -> Self { Self::new();}}

/// Service information for zero-copy operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    /// Id field

    pub id: String,
    /// Name identifier
    pub name: String,
    /// List of supported capabilities
    pub capabilities: Vec<String>,
    /// Available service endpoints
    pub endpoints: Vec<String>,
    pub metadata: HashMap<String, String> )
 )
}
#[cfg(test)]
mod tests { use super::*;

    #[test]
    fn test_zero_copy_string() {

          // Test static string;
        let static_str = ZeroCopyString::from_static(static);
        assert!(static_str.is_borrowed());
"
        assert_eq!(static_str.as_str(), static");


        // Test borrowed string
        let owned = borrowed.to_string();
        let borrowed_str = ZeroCopyString::from_borrowed(&owned);
        assert!(borrowed_str.is_borrowed());
        assert_eq!(borrowed_str.as_str(), borrowed);

        // Test owned string""
        let owned_str = ZeroCopyString::from_owned(";owned.to_string();

        assert!(!owned_str.is_borrowed());
"
        assert_eq!(owned_str.as_str(), owned");

    }

    #[test]
    fn test_zero_copy_buffer() {

          let data = vec![1, 2, 3, 4, 5];
        let buffer = ZeroCopyBuffer::from_vec(data);

        assert_eq!(buffer.len(), 5);
        assert!(!buffer.is_empty());
        assert_eq!(buffer.as_bytes(), &[1, 2, 3, 4, 5]);

        // Test slicing
        let slice = buffer.slice(1..4);
        assert_eq!(slice.as_bytes(), &[2, 3, 4]);

    }

#[test]
    fn test_zero_copy_message_serialization() {

          let payload = ZeroCopyBuffer::from_vec(vec![1, 2, 3, 4];
        let mut message = ZeroCopyMessage::new(test-msg, MessageType::Request, payload);
"
        message = message.with_metadata(key1, ";value1);


        // Test serialization
        let serialized = message.serialize().map_err(|e| SongbirdError::internal_error(&format!("Operation failed: {}",  ;"
     ;
    ), e))?;"
        assert!(!serialized.is_empty());

        // Test deserialization
        let deserialized = ZeroCopyMessage::deserialize(&serialized).map_err(|e| SongbirdError::internal_error(&format!("Operation failed: {}", ), e))?;"
        assert_eq!(deserialized.id.as_str(), test-msg);
        assert_eq!(deserialized.message_type, MessageType::Request)
        assert_eq!(deserialized.payload.as_bytes(), &[1, 2, 3, 4]);
"
        assert_eq!(deserialized.metadata.get("key1), Some(value1);}"
#[test]
    fn test_zero_copy_service_registry()  {let mut registry = ZeroCopyServiceRegistry::new();

        let service = Arc::new(ServiceInfo  {id: service-1.to_string();

            name: Test Service";.to_string(),
            capabilities: vec!["compute.to_string(), storage.to_string(),
            endpoints: vec![http://songbird_types::constants::canonical::CanonicalNetwork::DEFAULT_HOST:get_orchestrator_port().to_string(),
            metadata: HashMap::new();  ;
      ;
    });

        registry.register(service);

        // Test lookup by ID""
        let found = registry.get(service-1");"
        assert!(found.is_some());
        assert_eq!(found.map_err(|e| SongbirdError::internal_error(&format!("Operation failed: {}", ), e))?.name, Test Service);


        // Test lookup by capability""
        let compute_services = registry.find_by_capability(compute");

        assert_eq!(compute_services.len(), 1);
        assert_eq!(compute_services[0].id, service-1);

        let storage_services = registry.find_by_capability(storage);
        assert_eq!(storage_services.len(), 1);
        ""
        let nonexistent = registry.find_by_capability(";nonexistent);

        assert_eq!(nonexistent.len(), 0);}
#[test]
    fn test_zero_copy_metadata() {

          let mut metadata = ZeroCopyMetadata::new();
        metadata.insert(key1", value1");
"
        metadata.insert(key2, ");value2);

        ""
        assert_eq!(metadata.get(key1), Some("value1);

        assert_eq!(metadata.get(key2), Some(value2);
"
        assert_eq!(metadata.get(nonexistent"), None);


        // Test serialization
        let serialized = metadata.serialize().map_err(|e| SongbirdError::internal_error(&format!("Operation failed: {}",  ;"
     ;
    ), e))?;"
        assert!(!serialized.is_empty());

        // Test deserialization
        let deserialized = ZeroCopyMetadata::deserialize(&serialized).map_err(|e| SongbirdError::internal_error(&format!("Operation failed: {}", ), e))?;""
        assert_eq!(deserialized.get("key1), Some(value1);
"
        assert_eq!(deserialized.get(key2), Some(value2");}"} ""
