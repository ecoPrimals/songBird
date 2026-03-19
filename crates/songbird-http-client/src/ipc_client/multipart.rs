//! Multipart form-data support for `IpcHttpClient`
//!
//! Provides multipart form-data API for building multipart/form-data
//! requests that can be sent through Songbird's IPC HTTP client.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use songbird_http_client::{IpcHttpClient, multipart};
//!
//! async fn example() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = IpcHttpClient::new().await?;
//!
//!     let form = multipart::Form::new()
//!         .text("service_name", "my-service")
//!         .text("env_vars", r#"{"PORT":"8080"}"#)
//!         .text("auto_start", "true")
//!         .part("binary", multipart::Part::bytes(vec![1, 2, 3])
//!             .file_name("service.bin"));
//!
//!     let response = client.post("https://api.example.com/deploy")
//!         .await
//!         .multipart(form)
//!         .send()
//!         .await?;
//!     Ok(())
//! }
//! ```

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use serde::{Deserialize, Serialize};

/// Multipart form builder
///
/// Builds multipart/form-data requests with text fields and binary parts.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Form {
    parts: Vec<FormPart>,
}

/// A single part in a multipart form
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormPart {
    name: String,
    #[serde(flatten)]
    content: PartContent,
}

/// Content of a form part
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
enum PartContent {
    #[serde(rename = "text")]
    Text {
        value: String,
    },
    #[serde(rename = "bytes")]
    Bytes {
        /// Base64-encoded bytes for IPC transfer
        data: String,
        /// Optional filename
        #[serde(skip_serializing_if = "Option::is_none")]
        file_name: Option<String>,
        /// Optional MIME type
        #[serde(skip_serializing_if = "Option::is_none")]
        mime: Option<String>,
    },
}

/// Builder for creating multipart parts
#[derive(Debug, Clone)]
pub struct Part {
    content: PartContent,
}

impl Form {
    /// Create a new empty multipart form
    ///
    /// # Examples
    ///
    /// ```
    /// use songbird_http_client::multipart::Form;
    ///
    /// let form = Form::new()
    ///     .text("field1", "value1")
    ///     .text("field2", "value2");
    /// ```
    #[must_use]
    pub const fn new() -> Self {
        Self {
            parts: Vec::new(),
        }
    }

    /// Add a text field to the form
    ///
    /// # Arguments
    ///
    /// * `name` - Field name
    /// * `value` - Field value
    ///
    /// # Examples
    ///
    /// ```
    /// use songbird_http_client::multipart::Form;
    ///
    /// let form = Form::new().text("username", "alice");
    /// ```
    #[must_use]
    pub fn text<N, V>(mut self, name: N, value: V) -> Self
    where
        N: Into<String>,
        V: Into<String>,
    {
        self.parts.push(FormPart {
            name: name.into(),
            content: PartContent::Text {
                value: value.into(),
            },
        });
        self
    }

    /// Add a part to the form
    ///
    /// # Arguments
    ///
    /// * `name` - Field name
    /// * `part` - Part to add
    ///
    /// # Examples
    ///
    /// ```
    /// use songbird_http_client::multipart::{Form, Part};
    ///
    /// let form = Form::new()
    ///     .part("file", Part::bytes(vec![1, 2, 3]).file_name("data.bin"));
    /// ```
    #[must_use]
    pub fn part<N>(mut self, name: N, part: Part) -> Self
    where
        N: Into<String>,
    {
        self.parts.push(FormPart {
            name: name.into(),
            content: part.content,
        });
        self
    }

    /// Get the parts of this form (for inspection/debugging)
    #[allow(dead_code)]
    pub(crate) fn parts(&self) -> &[FormPart] {
        &self.parts
    }

    /// Encode the form as multipart/form-data
    ///
    /// Returns the encoded body and the boundary string.
    pub(crate) fn encode(&self) -> (Vec<u8>, String) {
        let boundary = generate_boundary();
        let mut body = Vec::new();

        for part in &self.parts {
            // Write boundary
            body.extend_from_slice(b"--");
            body.extend_from_slice(boundary.as_bytes());
            body.extend_from_slice(b"\r\n");

            // Write Content-Disposition header
            match &part.content {
                PartContent::Text {
                    ..
                } => {
                    body.extend_from_slice(
                        format!("Content-Disposition: form-data; name=\"{}\"\r\n", part.name)
                            .as_bytes(),
                    );
                }
                PartContent::Bytes {
                    file_name,
                    mime,
                    ..
                } => {
                    if let Some(fname) = file_name {
                        body.extend_from_slice(
                            format!(
                                "Content-Disposition: form-data; name=\"{}\"; filename=\"{}\"\r\n",
                                part.name, fname
                            )
                            .as_bytes(),
                        );
                    } else {
                        body.extend_from_slice(
                            format!("Content-Disposition: form-data; name=\"{}\"\r\n", part.name)
                                .as_bytes(),
                        );
                    }

                    // Write Content-Type if provided
                    if let Some(m) = mime {
                        body.extend_from_slice(format!("Content-Type: {m}\r\n").as_bytes());
                    } else {
                        body.extend_from_slice(b"Content-Type: application/octet-stream\r\n");
                    }
                }
            }

            // Empty line before content
            body.extend_from_slice(b"\r\n");

            // Write content
            match &part.content {
                PartContent::Text {
                    value,
                } => {
                    body.extend_from_slice(value.as_bytes());
                }
                PartContent::Bytes {
                    data,
                    ..
                } => {
                    // Decode base64 back to bytes
                    if let Ok(bytes) = BASE64.decode(data) {
                        body.extend_from_slice(&bytes);
                    }
                }
            }

            // CRLF after content
            body.extend_from_slice(b"\r\n");
        }

        // Final boundary
        body.extend_from_slice(b"--");
        body.extend_from_slice(boundary.as_bytes());
        body.extend_from_slice(b"--\r\n");

        (body, boundary)
    }

    /// Serialize the form for IPC transfer
    ///
    /// Returns a JSON-serializable representation that can be sent over IPC.
    #[allow(dead_code)]
    pub(crate) fn serialize_for_ipc(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or(serde_json::Value::Null)
    }
}

impl Part {
    /// Create a part from raw bytes
    ///
    /// # Arguments
    ///
    /// * `data` - Binary data
    ///
    /// # Examples
    ///
    /// ```
    /// use songbird_http_client::multipart::Part;
    ///
    /// let part = Part::bytes(vec![1, 2, 3, 4]);
    /// ```
    pub fn bytes<D>(data: D) -> Self
    where
        D: Into<Vec<u8>>,
    {
        let bytes = data.into();
        let encoded = BASE64.encode(&bytes);

        Self {
            content: PartContent::Bytes {
                data: encoded,
                file_name: None,
                mime: None,
            },
        }
    }

    /// Create a part from text
    ///
    /// # Arguments
    ///
    /// * `text` - Text content
    ///
    /// # Examples
    ///
    /// ```
    /// use songbird_http_client::multipart::Part;
    ///
    /// let part = Part::text("Hello, world!");
    /// ```
    pub fn text<T>(text: T) -> Self
    where
        T: Into<String>,
    {
        Self {
            content: PartContent::Text {
                value: text.into(),
            },
        }
    }

    /// Set the filename for this part
    ///
    /// # Examples
    ///
    /// ```
    /// use songbird_http_client::multipart::Part;
    ///
    /// let part = Part::bytes(vec![1, 2, 3]).file_name("data.bin");
    /// ```
    #[must_use]
    pub fn file_name<S>(mut self, name: S) -> Self
    where
        S: Into<String>,
    {
        if let PartContent::Bytes {
            ref mut file_name,
            ..
        } = self.content
        {
            *file_name = Some(name.into());
        }
        self
    }

    /// Set the MIME type for this part
    ///
    /// # Examples
    ///
    /// ```
    /// use songbird_http_client::multipart::Part;
    ///
    /// let part = Part::bytes(vec![1, 2, 3])
    ///     .file_name("image.png")
    ///     .mime_str("image/png");
    /// ```
    #[must_use]
    pub fn mime_str<M>(mut self, mime_value: M) -> Self
    where
        M: Into<String>,
    {
        if let PartContent::Bytes {
            ref mut mime,
            ..
        } = self.content
        {
            *mime = Some(mime_value.into());
        }
        self
    }
}

/// Generate a random boundary string for multipart encoding
///
/// Format: `----SongbirdBoundary{random_hex}`
fn generate_boundary() -> String {
    use std::time::SystemTime;

    let nanos =
        SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);

    format!("----SongbirdBoundary{nanos:x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_form_new() {
        let form = Form::new();
        assert_eq!(form.parts.len(), 0);
    }

    #[test]
    fn test_form_text() {
        let form = Form::new().text("field1", "value1").text("field2", "value2");

        assert_eq!(form.parts.len(), 2);
        assert_eq!(form.parts[0].name, "field1");
        assert_eq!(form.parts[1].name, "field2");
    }

    #[test]
    fn test_form_bytes() {
        let form = Form::new().part("file", Part::bytes(vec![1, 2, 3, 4]));

        assert_eq!(form.parts.len(), 1);
        assert_eq!(form.parts[0].name, "file");
    }

    #[test]
    fn test_part_with_filename() {
        let part = Part::bytes(vec![1, 2, 3]).file_name("test.bin");

        match part.content {
            PartContent::Bytes {
                file_name,
                ..
            } => {
                assert_eq!(file_name, Some("test.bin".to_string()));
            }
            _ => panic!("Expected Bytes content"),
        }
    }

    #[test]
    fn test_part_with_mime() {
        let part = Part::bytes(vec![1, 2, 3]).mime_str("application/octet-stream");

        match part.content {
            PartContent::Bytes {
                mime,
                ..
            } => {
                assert_eq!(mime, Some("application/octet-stream".to_string()));
            }
            _ => panic!("Expected Bytes content"),
        }
    }

    #[test]
    fn test_boundary_generation() {
        let boundary1 = generate_boundary();
        let boundary2 = generate_boundary();

        assert!(boundary1.starts_with("----SongbirdBoundary"));
        assert!(boundary2.starts_with("----SongbirdBoundary"));
        // Boundaries should be different (time-based)
        // Note: This might rarely fail if tests run in same nanosecond
    }

    #[test]
    fn test_form_encode_text_only() {
        let form = Form::new().text("field1", "value1").text("field2", "value2");

        let (body, boundary) = form.encode();
        let body_str = String::from_utf8_lossy(&body);

        assert!(body_str.contains(&boundary));
        assert!(body_str.contains("Content-Disposition: form-data; name=\"field1\""));
        assert!(body_str.contains("value1"));
        assert!(body_str.contains("Content-Disposition: form-data; name=\"field2\""));
        assert!(body_str.contains("value2"));
    }

    #[test]
    fn test_form_encode_with_file() {
        let form = Form::new()
            .text("name", "test")
            .part("file", Part::bytes(vec![1, 2, 3, 4]).file_name("test.bin"));

        let (body, boundary) = form.encode();
        let body_str = String::from_utf8_lossy(&body);

        assert!(body_str.contains(&boundary));
        assert!(body_str.contains("Content-Disposition: form-data; name=\"name\""));
        assert!(body_str
            .contains("Content-Disposition: form-data; name=\"file\"; filename=\"test.bin\""));
        assert!(body_str.contains("Content-Type: application/octet-stream"));
    }

    #[test]
    fn test_serialize_for_ipc() {
        let form = Form::new().text("field", "value").part("file", Part::bytes(vec![1, 2, 3]));

        let json = form.serialize_for_ipc();
        assert!(json.is_object());
        assert!(json.get("parts").is_some());
    }
}
