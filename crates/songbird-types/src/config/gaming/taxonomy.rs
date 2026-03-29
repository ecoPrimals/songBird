// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Gaming taxonomy: modes, types, and protocol classification.

use serde::{Deserialize, Serialize};

/// Gaming mode enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GamingMode {
    /// Performance optimized mode
    Performance,
    /// Balanced mode
    Balanced,
    /// Power saving mode
    PowerSaver,
}

/// Game type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameType {
    /// First-person shooter
    Fps,
    /// Real-time strategy
    Rts,
    /// Multiplayer online battle arena
    Moba,
    /// Role-playing game
    Rpg,
    /// Custom game type
    Custom(String),
}

/// Performance mode enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceMode {
    /// Low latency, high CPU usage
    HighPerformance,
    /// Balanced performance and resource usage
    Balanced,
    /// Low resource usage, higher latency
    PowerSaver,
}

/// Game protocol classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum GameProtocolClass {
    /// Real-time strategy games (e.g., `StarCraft`, `Age of Empires`)
    RealTimeStrategy,
    /// First-person shooter games (e.g., `Quake`, `Doom`)
    FirstPersonShooter,
    /// Multiplayer online battle arena (e.g., `DOTA`, `LoL`)
    MultiplayerOnlineBattleArena,
    /// Massively multiplayer online games
    MassivelyMultiplayerOnline,
    /// Turn-based strategy games
    TurnBasedStrategy,
    /// Racing games
    Racing,
    /// Sports games
    Sports,
    /// Custom protocol
    Custom(String),
}
