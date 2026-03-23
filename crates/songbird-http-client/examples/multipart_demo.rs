// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Example: Using multipart/form-data with `IpcHttpClient`
//!
//! This example demonstrates how to use the multipart API to upload files
//! and text data through Songbird's IPC HTTP client.

use songbird_http_client::{IpcHttpClient, multipart};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "info,songbird_http_client=debug".to_string()),
        )
        .init();

    println!("🧪 Multipart Form Data Example");
    println!("════════════════════════════════");

    // Create client (connects to Songbird via IPC)
    let _client = IpcHttpClient::new().await?;
    println!("✅ Connected to Songbird IPC");

    // Example 1: Simple multipart form with text fields
    println!("\n📝 Example 1: Text-only multipart form");
    let _form1 = multipart::Form::new()
        .text("username", "alice")
        .text("email", "alice@example.com")
        .text("message", "Hello from Songbird!");

    println!("   Created form with 3 text fields");
    println!("   Fields: username, email, message");

    // Example 2: Multipart form with file upload
    println!("\n📦 Example 2: File upload with multipart");

    // Simulate binary data (could be from fs::read)
    let binary_data = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG header
        0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
    ];

    let _form2 = multipart::Form::new()
        .text("service_name", "my-service")
        .text("description", "A test service deployment")
        .part(
            "binary",
            multipart::Part::bytes(binary_data.clone())
                .file_name("service.bin")
                .mime_str("application/octet-stream"),
        );

    println!("   Created form with file upload:");
    println!("   - Text: service_name, description");
    println!("   - File: service.bin ({} bytes)", binary_data.len());

    // Example 3: Multiple files
    println!("\n📁 Example 3: Multiple file uploads");

    let config_data = b"{\"port\": 8080, \"host\": \"0.0.0.0\"}";
    let readme_data = b"# My Service\n\nThis is a test service.";

    let _form3 = multipart::Form::new()
        .text("project_name", "test-project")
        .part(
            "binary",
            multipart::Part::bytes(binary_data)
                .file_name("app.bin")
                .mime_str("application/octet-stream"),
        )
        .part(
            "config",
            multipart::Part::bytes(config_data.to_vec())
                .file_name("config.json")
                .mime_str("application/json"),
        )
        .part(
            "readme",
            multipart::Part::bytes(readme_data.to_vec())
                .file_name("README.md")
                .mime_str("text/markdown"),
        );

    println!("   Created form with 3 files:");
    println!("   - app.bin (application/octet-stream)");
    println!("   - config.json (application/json)");
    println!("   - README.md (text/markdown)");

    // Note: Uncomment to actually send (requires a test server)
    /*
    println!("\n🚀 Sending multipart request...");
    let response = client
        .post("https://httpbin.org/post")
        .multipart(form3)
        .send()
        .await?;

    println!("✅ Response status: {}", response.status());
    println!("📄 Response body:");
    println!("{}", response.text().await?);
    */

    println!("\n✅ Multipart API demonstration complete!");
    println!("   All forms created successfully.");
    println!("   Ready for real HTTP requests.");

    println!("\n💡 Usage Pattern:");
    println!("   1. Create form: multipart::Form::new()");
    println!("   2. Add fields: .text(name, value)");
    println!("   3. Add files: .part(name, Part::bytes(...).file_name(...))");
    println!("   4. Send: client.post(url).multipart(form).send().await");

    Ok(())
}
