//! System thresholds the user may tune in `system.toml` in Codync's data folder (`~/.codync/system.toml`). Every
//! field defaults to Codync's own value, so the file only names what it changes, and a missing file changes nothing.
//!
//! ```toml
//! [compaction]
//! turn_end_cap = 150000
//!
//! [team]
//! max_request_hops = 4
//! ```

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

const FILE: &str = "system.toml";

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct SystemConfig {
    /// When to compact a conversation or roll it over (`codync_mem::compaction`).
    pub compaction: codync_mem::compaction::Config,
    /// Bot messages: the chain's hop limit and how many may wait.
    pub team: TeamLimits,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
#[allow(clippy::struct_field_names, reason = "the names users write in system.toml")]
pub struct TeamLimits {
    /// Handoffs in one chain of bot requests before the user must start another.
    pub max_request_hops: u8,
    /// Bot messages waiting across all bots.
    pub max_pending_messages: usize,
    /// Bot messages one bot may have waiting.
    pub max_pending_messages_per_sender: usize,
}

impl TeamLimits {
    pub const DEFAULT: Self =
        Self { max_request_hops: 8, max_pending_messages: 64, max_pending_messages_per_sender: 16 };
}

impl Default for TeamLimits {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl SystemConfig {
    /// `system.toml` in Codync's data folder. A file that can't be read or parsed is logged and ignored: the host
    /// still starts, with the defaults.
    pub fn load() -> Self {
        let path = crate::service::data_dir().join(FILE);
        match Self::read(&path) {
            Ok(Some(config)) => {
                tracing::info!(path = %path.display(), "loaded system settings");
                config
            }
            Ok(None) => Self::default(),
            Err(error) => {
                tracing::warn!(path = %path.display(), error = format!("{error:#}"), "ignoring system settings");
                Self::default()
            }
        }
    }

    fn read(path: &Path) -> Result<Option<Self>> {
        let text = match std::fs::read_to_string(path) {
            Ok(text) => text,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error).context("couldn't read it"),
        };
        Self::parse(&text).map(Some)
    }

    fn parse(text: &str) -> Result<Self> {
        toml::from_str(text).context("couldn't parse it")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_empty_file_keeps_every_default() {
        assert_eq!(SystemConfig::parse("").unwrap(), SystemConfig::default());
    }

    #[test]
    fn a_file_overrides_only_what_it_names() {
        let config = SystemConfig::parse("[compaction]\nper_day = 5\n\n[team]\nmax_request_hops = 3\n").unwrap();
        assert_eq!(config.compaction.per_day, 5);
        assert_eq!(config.compaction.large, codync_mem::compaction::Config::default().large);
        assert_eq!(config.team, TeamLimits { max_request_hops: 3, ..TeamLimits::default() });
    }

    #[test]
    fn a_misspelled_setting_is_an_error_not_a_silent_default() {
        assert!(SystemConfig::parse("[team]\nmax_hops = 3\n").is_err());
    }

    #[test]
    fn a_missing_file_is_no_error() {
        let path = std::env::temp_dir().join(format!("codync-system-{}.toml", uuid::Uuid::new_v4()));
        assert_eq!(SystemConfig::read(&path).unwrap(), None);
    }
}
