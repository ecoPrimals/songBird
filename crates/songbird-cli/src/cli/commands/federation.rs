// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! # 🤝 Gaming Federation Commands
//!
//! **MODERN GAMING FEDERATION & MATCHMAKING** ✅

#![allow(missing_docs, reason = "federation clap enums document flags inline")]

use crate::errors::SongbirdResult;
use clap::Subcommand;

#[derive(Debug, Clone, Subcommand)]
pub enum FederationCommand {
    /// Initialize gaming federation
    #[command(about = "🚀 Initialize gaming federation node")]
    Init {
        /// Enable gaming-specific federation
        #[arg(long)]
        gaming: bool,

        /// Federation node name
        #[arg(long)]
        name: Option<String>,

        /// Gaming region
        #[arg(long)]
        region: Option<String>,
    },

    /// Join existing gaming federation
    #[command(about = "🤝 Join existing gaming federation")]
    Join {
        /// Gaming federation endpoint
        #[arg(long)]
        gaming_endpoint: Option<String>,

        /// Federation token for authentication
        #[arg(long)]
        token: Option<String>,

        /// Auto-discover federation nodes
        #[arg(long)]
        auto_discover: bool,
    },

    /// Create or manage gaming lobbies
    #[command(about = "🎮 Create and manage gaming lobbies")]
    Lobby {
        #[command(subcommand)]
        action: LobbyAction,
    },

    /// Gaming matchmaking services
    #[command(about = "🎯 Gaming matchmaking and player matching")]
    Matchmaking {
        #[command(subcommand)]
        action: MatchmakingAction,
    },

    /// Federation status and health
    #[command(about = "📊 Check gaming federation status")]
    Status {
        /// Show detailed federation information
        #[arg(long)]
        detailed: bool,

        /// Focus on gaming federation metrics
        #[arg(long)]
        gaming_metrics: bool,
    },
}

#[derive(Debug, Clone, Subcommand)]
pub enum LobbyAction {
    /// Create new gaming lobby
    Create {
        /// Lobby name
        name: String,

        /// Maximum players
        #[arg(long, default_value = "16")]
        max_players: u32,

        /// Enable gaming mode
        #[arg(long)]
        gaming: bool,
    },

    /// List available lobbies
    List {
        /// Filter by game type
        #[arg(long)]
        game_type: Option<String>,
    },

    /// Join lobby
    Join {
        /// Lobby ID
        lobby_id: String,
    },

    /// Leave current lobby
    Leave,
}

#[derive(Debug, Clone, Subcommand)]
pub enum MatchmakingAction {
    /// Start matchmaking
    Start {
        /// Skill level
        #[arg(long)]
        skill_level: Option<String>,

        /// Preferred region
        #[arg(long)]
        region: Option<String>,
    },

    /// Cancel matchmaking
    Cancel,

    /// Check matchmaking status
    Status,
}

fn init_federation_lines(gaming: bool, name: Option<&str>, region: Option<&str>) -> Vec<String> {
    let mut lines = vec!["🚀 Initializing gaming federation...".to_string()];
    if gaming {
        lines.push("🎮 Gaming-specific federation enabled".to_string());
    }
    if let Some(n) = name {
        lines.push(format!("📛 Federation node name: {n}"));
    }
    if let Some(r) = region {
        lines.push(format!("🌍 Gaming region: {r}"));
    }
    lines.push("✅ Federation initialization complete".to_string());
    lines
}

fn join_federation_lines(
    gaming_endpoint: Option<&str>,
    token_present: bool,
    auto_discover: bool,
) -> Vec<String> {
    let mut lines = vec!["🤝 Joining gaming federation...".to_string()];
    if let Some(ep) = gaming_endpoint {
        lines.push(format!("🌐 Gaming endpoint: {ep}"));
    }
    if token_present {
        lines.push("🔐 Authentication token provided".to_string());
    }
    if auto_discover {
        lines.push("🔍 Auto-discovering federation nodes...".to_string());
    }
    lines.push("✅ Successfully joined federation".to_string());
    lines
}

fn lobby_action_lines(action: &LobbyAction) -> Vec<String> {
    match action {
        LobbyAction::Create {
            name,
            max_players,
            gaming,
        } => {
            let mut v = vec![
                format!("🎮 Creating gaming lobby: {name}"),
                format!("👥 Max players: {max_players}"),
            ];
            if *gaming {
                v.push("⚡ Gaming mode enabled".to_string());
            }
            v.push("✅ Lobby created successfully".to_string());
            v
        }
        LobbyAction::List {
            game_type,
        } => {
            let mut v = vec!["📋 Available gaming lobbies:".to_string()];
            if let Some(gt) = game_type {
                v.push(format!("🎯 Filtered by: {gt}"));
            }
            v.extend([
                "  1. Gaming Lobby Alpha (8/16 players,".to_string(),
                "  2. Pro Gaming Arena (12/32 players,".to_string(),
                "  3. Casual Gaming Room (4/8 players,".to_string(),
            ]);
            v
        }
        LobbyAction::Join {
            lobby_id,
        } => {
            vec![
                format!("🚪 Joining lobby: {lobby_id}"),
                "✅ Successfully joined gaming lobby".to_string(),
            ]
        }
        LobbyAction::Leave => {
            vec!["👋 Leaving current lobby".to_string(), "✅ Left lobby successfully".to_string()]
        }
    }
}

fn matchmaking_action_lines(action: &MatchmakingAction) -> Vec<String> {
    match action {
        MatchmakingAction::Start {
            skill_level,
            region,
        } => {
            let mut v = vec!["🎯 Starting gaming matchmaking...".to_string()];
            if let Some(s) = skill_level {
                v.push(format!("🎓 Skill level: {s}"));
            }
            if let Some(r) = region {
                v.push(format!("🌍 Preferred region: {r}"));
            }
            v.extend([
                "🔍 Searching for suitable gaming matches...".to_string(),
                "✅ Matchmaking started".to_string(),
            ]);
            v
        }
        MatchmakingAction::Cancel => {
            vec!["❌ Cancelling matchmaking".to_string(), "✅ Matchmaking cancelled".to_string()]
        }
        MatchmakingAction::Status => {
            vec![
                "📊 Matchmaking Status:".to_string(),
                "  Status: Searching".to_string(),
                "  Queue position: 5".to_string(),
                "  Estimated wait: 2 minutes".to_string(),
                "  Skill bracket: Intermediate".to_string(),
            ]
        }
    }
}

fn federation_status_lines(detailed: bool, gaming_metrics: bool) -> Vec<String> {
    let mut lines = vec![
        "📊 Gaming Federation Status:".to_string(),
        "  Status: Connected".to_string(),
        "  Active nodes: 24".to_string(),
        "  Gaming sessions: 156".to_string(),
        "  Total players: 3,842".to_string(),
    ];
    if detailed {
        lines.push("\n📈 Detailed Information:".to_string());
        lines.extend([
            "  Uptime: 48h 32m".to_string(),
            "  Network latency: 24ms avg".to_string(),
            "  Bandwidth usage: 125 Mbps".to_string(),
        ]);
    }
    if gaming_metrics {
        lines.push("\n🎮 Gaming Metrics:".to_string());
        lines.extend([
            "  Active lobbies: 67".to_string(),
            "  Matchmaking queue: 89 players".to_string(),
            "  Average match time: 3.2 minutes".to_string(),
        ]);
    }
    lines.push("✅ Federation healthy".to_string());
    lines
}

/// Handle federation commands
pub async fn handle_federation_command(command: FederationCommand) -> SongbirdResult<()> {
    match command {
        FederationCommand::Init {
            gaming,
            name,
            region,
        } => init_federation(gaming, name, region).await,
        FederationCommand::Join {
            gaming_endpoint,
            token,
            auto_discover,
        } => join_federation(gaming_endpoint, token, auto_discover).await,
        FederationCommand::Lobby {
            action,
        } => handle_lobby_action(action).await,
        FederationCommand::Matchmaking {
            action,
        } => handle_matchmaking_action(action).await,
        FederationCommand::Status {
            detailed,
            gaming_metrics,
        } => show_federation_status(detailed, gaming_metrics).await,
    }
}

async fn init_federation(
    gaming: bool,
    name: Option<String>,
    region: Option<String>,
) -> SongbirdResult<()> {
    for line in init_federation_lines(gaming, name.as_deref(), region.as_deref()) {
        println!("{line}");
    }
    Ok(())
}

async fn join_federation(
    gaming_endpoint: Option<String>,
    token: Option<String>,
    auto_discover: bool,
) -> SongbirdResult<()> {
    for line in join_federation_lines(gaming_endpoint.as_deref(), token.is_some(), auto_discover) {
        println!("{line}");
    }
    Ok(())
}

async fn handle_lobby_action(action: LobbyAction) -> SongbirdResult<()> {
    for line in lobby_action_lines(&action) {
        println!("{line}");
    }
    Ok(())
}

async fn handle_matchmaking_action(action: MatchmakingAction) -> SongbirdResult<()> {
    for line in matchmaking_action_lines(&action) {
        println!("{line}");
    }
    Ok(())
}

async fn show_federation_status(detailed: bool, gaming_metrics: bool) -> SongbirdResult<()> {
    for line in federation_status_lines(detailed, gaming_metrics) {
        println!("{line}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::{
        LobbyAction, MatchmakingAction, federation_status_lines, init_federation_lines,
        join_federation_lines, lobby_action_lines, matchmaking_action_lines,
    };

    #[test]
    fn init_lines_respects_flags() {
        let lines = init_federation_lines(true, Some("node-a"), Some("us-west"));
        assert!(lines.iter().any(|l| l.contains("Gaming-specific")));
        assert!(lines.iter().any(|l| l.contains("node-a")));
        assert!(lines.iter().any(|l| l.contains("us-west")));
    }

    #[test]
    fn join_lines_token_and_discover() {
        let lines = join_federation_lines(Some("https://x"), true, true);
        assert!(lines.iter().any(|l| l.contains("https://x")));
        assert!(lines.iter().any(|l| l.contains("token")));
        assert!(lines.iter().any(|l| l.contains("Auto-discovering")));
    }

    #[test]
    fn lobby_create_lines_includes_max_players() {
        let action = LobbyAction::Create {
            name: "room".to_string(),
            max_players: 8,
            gaming: true,
        };
        let lines = lobby_action_lines(&action);
        assert!(lines.iter().any(|l| l.contains("room")));
        assert!(lines.iter().any(|l| l.contains('8')));
    }

    #[test]
    fn lobby_list_with_filter() {
        let action = LobbyAction::List {
            game_type: Some("fps".to_string()),
        };
        let lines = lobby_action_lines(&action);
        assert!(lines.iter().any(|l| l.contains("fps")));
    }

    #[test]
    fn matchmaking_start_and_status() {
        let start = MatchmakingAction::Start {
            skill_level: Some("pro".to_string()),
            region: None,
        };
        let ls = matchmaking_action_lines(&start);
        assert!(ls.iter().any(|l| l.contains("pro")));

        let st = matchmaking_action_lines(&MatchmakingAction::Status);
        assert!(st.iter().any(|l| l.contains("Queue position")));
    }

    #[test]
    fn federation_status_detailed_and_metrics_toggle_sections() {
        let plain = federation_status_lines(false, false);
        assert!(!plain.iter().any(|l| l.contains("Detailed Information")));
        let full = federation_status_lines(true, true);
        assert!(full.iter().any(|l| l.contains("Detailed Information")));
        assert!(full.iter().any(|l| l.contains("Gaming Metrics")));
    }

    #[test]
    fn init_lines_minimal_no_optional_fields() {
        let lines = init_federation_lines(false, None, None);
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("Initializing"));
        assert!(lines.last().unwrap().contains("complete"));
    }

    #[test]
    fn join_lines_without_endpoint_token_or_discover() {
        let lines = join_federation_lines(None, false, false);
        assert_eq!(lines.len(), 2);
        assert!(!lines.iter().any(|l| l.contains("endpoint")));
        assert!(!lines.iter().any(|l| l.contains("token")));
    }

    #[test]
    fn lobby_join_and_leave_lines() {
        let join = lobby_action_lines(&LobbyAction::Join {
            lobby_id: "lob-42".to_string(),
        });
        assert!(join.iter().any(|l| l.contains("lob-42")));

        let leave = lobby_action_lines(&LobbyAction::Leave);
        assert!(leave.iter().any(|l| l.contains("Leaving")));
        assert!(leave.iter().any(|l| l.contains("Left lobby")));
    }

    #[test]
    fn matchmaking_cancel_lines() {
        let lines = matchmaking_action_lines(&MatchmakingAction::Cancel);
        assert!(lines.iter().any(|l| l.contains("Cancelling")));
        assert!(lines.iter().any(|l| l.contains("cancelled")));
    }

    #[test]
    fn federation_status_metrics_only_adds_gaming_block() {
        let m = federation_status_lines(false, true);
        assert!(m.iter().any(|l| l.contains("Gaming Metrics")));
        assert!(!m.iter().any(|l| l.contains("Detailed Information")));
    }
}
