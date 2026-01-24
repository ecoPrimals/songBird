//! HTTPS request example (requires BearDog running)

use songbird_http_client::SongbirdHttpClient;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt().with_max_level(tracing::Level::DEBUG).init();

    // Create client
    let beardog_socket =
        std::env::var("BEARDOG_SOCKET").unwrap_or_else(|_| "/tmp/beardog-nat0.sock".to_string());

    println!("Using BearDog at: {}", beardog_socket);

    let client = SongbirdHttpClient::new(beardog_socket);

    // Make HTTPS request
    println!("\n🔒 Making HTTPS request to example.com...\n");

    let response = client.request("GET", "https://example.com", HashMap::new(), None).await?;

    println!("\n✅ Response received!");
    println!("Status: {}", response.status);
    println!("Headers: {:#?}", response.headers);
    println!(
        "Body preview: {}",
        serde_json::to_string(&response.body)?.chars().take(200).collect::<String>()
    );

    Ok(())
}
