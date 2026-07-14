//! Configuration loading and env-var overrides.
//!
//! The on-disk schema mirrors the original `config.json` used by the
//! PowerShell implementation exactly, so an existing configuration keeps
//! working unchanged. Individual values can additionally be overridden via
//! environment variables (documented in the README).

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceConfig {
    /// Host/IP to ping (the OnePlus 9 Pro).
    pub target: String,
    /// Ping timeout in milliseconds.
    pub ping_timeout_ms: u32,
    /// Seconds to wait between presence checks in the continuous loop.
    pub check_interval_s: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicConfig {
    /// Length of the verification recording taken on every transition.
    pub verify_seconds: u32,
    /// Total time to wait for a spoken response after a greeting.
    pub response_timeout_s: u32,
    /// Pause after the greeting before listening (echo decay).
    pub response_cooldown_s: u64,
    /// Level (dB) above which a listening chunk counts as a "response".
    pub speech_threshold_db: f32,
    /// Length of each listening chunk.
    pub chunk_seconds: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsConfig {
    pub greeting_language: String,
    pub panic_language: String,
    pub greeting_text: String,
    pub panic_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub log_dir: String,
    pub audio_dir: String,
    pub transition_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub device: DeviceConfig,
    pub mic: MicConfig,
    pub tts: TtsConfig,
    pub logging: LoggingConfig,
}

impl Config {
    /// Parse a config from a JSON string (the same schema as `config.json`).
    pub fn from_json_str(s: &str) -> Result<Self> {
        let cfg: Config = serde_json::from_str(s).context("failed to parse config JSON")?;
        Ok(cfg)
    }

    /// Load the config from a file path, then apply environment overrides.
    pub fn load(path: &Path) -> Result<Self> {
        let raw = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read config file: {}", path.display()))?;
        let mut cfg = Self::from_json_str(&raw)?;
        cfg.apply_env_overrides();
        Ok(cfg)
    }

    /// Apply `PRESENCE_*` environment-variable overrides in place.
    ///
    /// Invalid values are ignored (the file value is kept) so a stray env
    /// var can never crash the monitor.
    pub fn apply_env_overrides(&mut self) {
        if let Ok(v) = std::env::var("PRESENCE_TARGET") {
            if !v.trim().is_empty() {
                self.device.target = v;
            }
        }
        if let Some(v) = env_parse::<u32>("PRESENCE_PING_TIMEOUT_MS") {
            self.device.ping_timeout_ms = v;
        }
        if let Some(v) = env_parse::<u64>("PRESENCE_CHECK_INTERVAL_S") {
            self.device.check_interval_s = v;
        }
        if let Some(v) = env_parse::<u32>("PRESENCE_VERIFY_SECONDS") {
            self.mic.verify_seconds = v;
        }
        if let Some(v) = env_parse::<u32>("PRESENCE_RESPONSE_TIMEOUT_S") {
            self.mic.response_timeout_s = v;
        }
        if let Some(v) = env_parse::<u64>("PRESENCE_RESPONSE_COOLDOWN_S") {
            self.mic.response_cooldown_s = v;
        }
        if let Some(v) = env_parse::<f32>("PRESENCE_SPEECH_THRESHOLD_DB") {
            self.mic.speech_threshold_db = v;
        }
        if let Some(v) = env_parse::<u32>("PRESENCE_CHUNK_SECONDS") {
            self.mic.chunk_seconds = v;
        }
    }

    /// Basic sanity checks used by `self-check`.
    pub fn validate(&self) -> Vec<String> {
        let mut problems = Vec::new();
        if self.device.target.trim().is_empty() {
            problems.push("device.target is empty".to_string());
        }
        if self.device.check_interval_s == 0 {
            problems.push("device.check_interval_s must be > 0".to_string());
        }
        if self.mic.chunk_seconds == 0 {
            problems.push("mic.chunk_seconds must be > 0".to_string());
        }
        if self.mic.verify_seconds == 0 {
            problems.push("mic.verify_seconds must be > 0".to_string());
        }
        problems
    }
}

fn env_parse<T: std::str::FromStr>(key: &str) -> Option<T> {
    std::env::var(key)
        .ok()
        .and_then(|v| v.trim().parse::<T>().ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"{
      "device": { "target": "oneplus9pro.local", "ping_timeout_ms": 2000, "check_interval_s": 30 },
      "mic": { "verify_seconds": 30, "response_timeout_s": 300, "response_cooldown_s": 12, "speech_threshold_db": -40.0, "chunk_seconds": 2 },
      "tts": { "greeting_language": "fr-FR", "panic_language": "de-DE", "greeting_text": "hi", "panic_text": "panic" },
      "logging": { "log_dir": "logs", "audio_dir": "logs/audio", "transition_dir": "logs/transitions" }
    }"#;

    #[test]
    fn parses_sample_config() {
        let cfg = Config::from_json_str(SAMPLE).unwrap();
        assert_eq!(cfg.device.target, "oneplus9pro.local");
        assert_eq!(cfg.device.ping_timeout_ms, 2000);
        assert_eq!(cfg.device.check_interval_s, 30);
        assert_eq!(cfg.mic.verify_seconds, 30);
        assert_eq!(cfg.mic.response_timeout_s, 300);
        assert_eq!(cfg.mic.response_cooldown_s, 12);
        assert!((cfg.mic.speech_threshold_db - (-40.0)).abs() < f32::EPSILON);
        assert_eq!(cfg.mic.chunk_seconds, 2);
        assert_eq!(cfg.tts.greeting_language, "fr-FR");
        assert_eq!(cfg.logging.audio_dir, "logs/audio");
    }

    #[test]
    fn validate_flags_bad_values() {
        let mut cfg = Config::from_json_str(SAMPLE).unwrap();
        assert!(cfg.validate().is_empty());
        cfg.device.target = "  ".to_string();
        cfg.device.check_interval_s = 0;
        let problems = cfg.validate();
        assert!(problems.iter().any(|p| p.contains("target")));
        assert!(problems.iter().any(|p| p.contains("check_interval_s")));
    }

    #[test]
    fn rejects_invalid_json() {
        assert!(Config::from_json_str("{ not json").is_err());
    }
}
