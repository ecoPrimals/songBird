// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

use crate::codec::bytes::{read_u8, read_u16, write_u8, write_u16};
use crate::error::{Result, TlsError};
use crate::messages::ContentType;
use crate::{MAX_RECORD_SIZE, TLS_VERSION_1_2};

use super::layer::RecordLayer;

impl RecordLayer {
    /// Frame a plaintext message into a TLS record
    ///
    /// This creates the 5-byte header + payload.
    /// Does NOT encrypt (encryption is handled separately).
    ///
    /// # Errors
    ///
    /// Returns an error if payload exceeds `MAX_RECORD_SIZE` or length truncation occurs.
    pub fn frame_plaintext(
        &mut self,
        content_type: ContentType,
        payload: &[u8],
    ) -> Result<Vec<u8>> {
        // Validate payload length
        if payload.len() > MAX_RECORD_SIZE {
            return Err(TlsError::RecordTooLarge {
                size: payload.len(),
            });
        }

        let mut record = Vec::with_capacity(5 + payload.len());

        // Content type (1 byte)
        write_u8(&mut record, content_type.into());

        // Legacy record version (2 bytes) - always 0x0303 (TLS 1.2) for compatibility
        write_u16(&mut record, TLS_VERSION_1_2);

        // Length (2 bytes)
        write_u16(
            &mut record,
            u16::try_from(payload.len()).map_err(|_| TlsError::RecordTooLarge {
                size: payload.len(),
            })?,
        );

        // Payload
        record.extend_from_slice(payload);

        Ok(record)
    }

    /// Parse a TLS record from bytes
    ///
    /// Returns: (`content_type`, payload, `bytes_consumed`)
    ///
    /// # Errors
    ///
    /// Returns an error if the buffer is too short, record is too large, or incomplete.
    pub fn parse_record(&mut self, buf: &[u8]) -> Result<(ContentType, Vec<u8>, usize)> {
        if buf.len() < 5 {
            return Err(TlsError::ProtocolError(
                "Record too short: need at least 5 bytes for header".to_string(),
            ));
        }

        let mut offset = 0;

        // Content type (1 byte)
        let content_type = ContentType::from(read_u8(buf, &mut offset)?);

        // Legacy record version (2 bytes) - we don't strictly validate this
        let _legacy_version = read_u16(buf, &mut offset)?;

        // Length (2 bytes)
        let length = read_u16(buf, &mut offset)? as usize;

        // Validate length
        if length > MAX_RECORD_SIZE {
            return Err(TlsError::RecordTooLarge {
                size: length,
            });
        }

        // Check if we have the full payload
        if offset + length > buf.len() {
            return Err(TlsError::ProtocolError(format!(
                "Incomplete record: need {} bytes, have {}",
                length,
                buf.len() - offset
            )));
        }

        // Extract payload
        let payload = buf[offset..offset + length].to_vec();
        offset += length;

        Ok((content_type, payload, offset))
    }
}
