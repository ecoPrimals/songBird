//! Configuration for Squirrel service

use std::env;

#[derive(Debug, Clone)]
pub struct SquirrelConfig {
    pub port: u16,
    pub ai_provider: String,
    pub anthropic_api_key: Option<String>,
    pub openai_api_key: Option<String>,
}

impl SquirrelConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        let port = env::var("SQUIRREL_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(9010);

        let ai_provider = env::var("AI_PROVIDER").unwrap_or_else(|_| "claude".to_string());

        let anthropic_api_key = env::var("ANTHROPIC_API_KEY")
            .ok()
            .or_else(|| {
                // Try loading from file
                std::fs::read_to_string(env::var("HOME").ok()? + "/.anthropic_api_key").ok()
            })
            .map(|s| s.trim().to_string());

        let openai_api_key = env::var("OPENAI_API_KEY")
            .ok()
            .or_else(|| {
                std::fs::read_to_string(env::var("HOME").ok()? + "/.openai_api_key").ok()
            })
            .map(|s| s.trim().to_string());

        Ok(Self {
            port,
            ai_provider,
            anthropic_api_key,
            openai_api_key,
        })
    }
}

