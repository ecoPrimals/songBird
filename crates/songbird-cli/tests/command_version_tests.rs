//! Tests for version command

use songbird_cli::cli::commands::version::{
    execute_version_command, show_detailed_version, show_simple_version,
};

#[tokio::test]
async fn test_execute_version_simple() {
    let result = execute_version_command(false).await;
    assert!(result.is_ok(), "Simple version command should succeed");
}

#[tokio::test]
async fn test_execute_version_detailed() {
    let result = execute_version_command(true).await;
    assert!(result.is_ok(), "Detailed version command should succeed");
}

#[tokio::test]
async fn test_show_simple_version() {
    let result = show_simple_version().await;
    assert!(result.is_ok(), "Show simple version should succeed");
}

#[tokio::test]
async fn test_show_detailed_version() {
    let result = show_detailed_version().await;
    assert!(result.is_ok(), "Show detailed version should succeed");
}
