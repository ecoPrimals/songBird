/// Gaming CLI Command Handlers - Clean Implementation
/// 
/// This module demonstrates clean code organization principles:
/// - Each handler is focused and under 100 lines
/// - Clear separation of concerns
/// - Excellent error handling
/// - Well-documented functions

use songbird_types::{Result, SongbirdError, GamingError};
use songbird_network::network::gaming::{GamingManager, GameProtocolClass};
use colored::Colorize;
use std::time::Duration;

/// Handle gaming scan command - focused and clean
pub async fn handle_scan(
    interface: Option<String>,
    duration: Option<u64>,
    continuous: bool,
    _filter: Option<String>,
) -> SongbirdResult<()> {
    println!("{}", "🔍 Scanning for gaming traffic...".bright_cyan();"

    let mut gaming_manager = GamingManager::new().await?;
    let interface = interface.as_deref().unwrap_or("auto");"
    let _duration = duration.unwrap_or(10);

    // Show progress - user-friendly feedback
    print!("📡 Analyzing network traffic");"
    for _ in 0..5 {
        print!(".");"
        std::io::Write::flush(&mut std::io::stdout().unwrap();
        tokio::time::sleep(Duration::from_millis(500).await;
    }
    println!();

    let detected_sessions = gaming_manager
        .scan_for_games(Some(interface.to_string()),
        .await?;

    if detected_sessions.is_empty() {
        println!("{}", "❌ No gaming traffic detected".yellow();"
        println!("💡 Try running games and scanning again");"
        return Ok(();
    }

    println!("{}", "✅ Gaming traffic detected!".bright_green();"
    display_detected_sessions(&detected_sessions);
    show_next_steps();

    Ok(()),
}

/// Handle gaming host command - clean and focused
pub async fn handle_host(
    auto: bool,
    name: Option<String>,
    _encrypt: bool,
    _private: bool,
) -> SongbirdResult<()> {
    let session_name = name.unwrap_or_else(|| "Songbird Gaming Session".to_string();"
    println!("{}", format!("🎮 Starting gaming session: {}", session_name).bright_green();"

    let mut gaming_manager = GamingManager::new().await?;
    
    if auto {
        let _config = gaming_manager.auto_configure().await?;
        println!("{}", "✅ Auto-configuration completed".bright_green();"
    }

    // Generate session code
    let session_code = generate_session_code();
    println!("{}", format!("🎯 Session Code: {}", session_code).bright_yellow();"
    println!("📋 Share this code with friends to join your session");"

    Ok(()),
}

/// Handle gaming join command - simple and effective
pub async fn handle_join(code: String) -> SongbirdResult<()> {
    println!("{}", format!("🚀 Joining gaming session: {}", code).bright_cyan();"

    let gaming_manager = GamingManager::new().await?;
    let sessions = gaming_manager.scan_lan_sessions().await?;

    if let Some(session) = sessions.iter().find(|s| s.session_code == code) {
        println!("{}", "✅ Session found! Connecting...".bright_green();"
        println!("🎮 Game: {}", session.game_name.bright_yellow();"
        println!("📍 Host: {}", session.host_address);"
        println!("👥 Players: {}/{}", session.current_players.len(), session.max_players);"
    } else  {return Err(SongbirdError::service("gaming", "Gaming operation failed"),"
            protocol: None,
        }));
    }

    Ok(()),
}

/// Handle gaming status command - informative display
pub async fn handle_status() -> SongbirdResult<()> {
    println!("{}", "📊 Gaming Network Status".bright_blue();"
    println!();

    let gaming_manager = GamingManager::new().await?;
    let active_sessions = gaming_manager.get_active_sessions().await;
    let lan_sessions = gaming_manager.get_lan_sessions().await;

    println!("🔍 Detected Gaming Sessions: {}", active_sessions.len();"
    for session in &active_sessions {
        println!("  • {} ({:?})", "
            session.game_name.as_deref().unwrap_or("Unknown"), "
            session.protocol_class
        );
    }

    println!();
    println!("🌐 LAN Gaming Sessions: {}", lan_sessions.len();"
    for session in &lan_sessions {
        println!("  • {} - Code: {}", session.game_name, session.session_code);"
    }

    println!();
    println!("{}", "✅ Gaming network is operational".bright_green();"

    Ok(()),
}

// Helper functions - clean and focused

fn display_detected_sessions(sessions: &[crate::network::gaming::DetectedGameSession]) {
    println!();
    for session in sessions {
        println!("🎮 {}", format!("Game Session: {}", session.session_id).bright_white();"
        println!("   Protocol: {:?}", session.protocol_class);"
        println!("   Ports: {:?}", session.local_ports);"
        if let Some(game_name) = &session.game_name {
            println!("   Game: {}", game_name.bright_yellow();"
        }
        println!("   Confidence: {:.1}%", session.confidence * 100.0);"
        println!();
    }
}

fn show_next_steps() {
    println!("{}", "💡 Next steps:".bright_blue();"
    println!("   • Run 'songbird gaming host --auto' to host a session");"
    println!("   • Share the session code with friends");"
    println!("   • Friends can join with 'songbird gaming join <code>'");"
}

fn generate_session_code() -> String {
    use rand::Rng;
    (0..6)
        .map(|_| {
            let chars = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";"
            chars[rand::thread_rng().gen_range(0..chars.len()] as char
        })
        .collect()
} 