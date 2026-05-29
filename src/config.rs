use crate::types::Mirror;
use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

// Include the JSON file at compile time
const MIRRORS_JSON: &str = include_str!("../assets/mirrors.json");

// ── Mirror config (mirrors.json) ──────────────────────────────────────────

/// Returns the path to the user mirrors config: `~/.config/tsm/mirrors.json`
pub fn get_config_path() -> Option<std::path::PathBuf> {
    ProjectDirs::from("", "", "tsm").map(|d| d.config_dir().join("mirrors.json"))
}

/// Load user-custom mirrors from `~/.config/tsm/mirrors.json`.
/// Returns an empty map when the file doesn't exist.
pub fn load_user_config() -> Result<HashMap<String, Vec<Mirror>>> {
    let path = match get_config_path() {
        Some(p) if p.exists() => p,
        _ => return Ok(HashMap::new()),
    };
    let content = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read user config: {:?}", path))?;
    serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse user config: {:?}", path))
}

/// Write user-custom mirrors to `~/.config/tsm/mirrors.json`.
/// Creates parent directories as needed.
pub fn save_user_config(map: &HashMap<String, Vec<Mirror>>) -> Result<()> {
    let path = get_config_path()
        .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create config dir: {:?}", parent))?;
    }
    let json = serde_json::to_string_pretty(map)?;
    fs::write(&path, json)
        .with_context(|| format!("Failed to write user config: {:?}", path))?;
    Ok(())
}

/// Ensure the user mirrors config file exists.
/// On first run (no config file), copies the built-in defaults to the user path.
/// On subsequent runs (config exists), does nothing — user changes survive upgrades.
pub fn ensure_user_config_initialized() -> Result<()> {
    let path = match get_config_path() {
        Some(p) => p,
        None => return Ok(()),
    };
    if path.exists() {
        return Ok(());
    }
    // Parse built-in defaults and save to user config path
    let builtin: HashMap<String, Vec<Mirror>> =
        serde_json::from_str(MIRRORS_JSON).unwrap_or_default();
    save_user_config(&builtin)?;
    Ok(())
}

/// Retrieve the mirror candidates for a given tool from the user config.
///
/// The user config (`~/.config/tsm/mirrors.json`) is the single source of truth.
/// On first install, `ensure_user_config_initialized()` copies the built-in
/// defaults there; afterwards the user owns the file and upgrades won't overwrite it.
pub fn get_candidates(tool_name: &str) -> Vec<Mirror> {
    match load_user_config() {
        Ok(user_config) => user_config.get(tool_name).cloned().unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

// ── Settings (settings.toml) ──────────────────────────────────────────────

const DEFAULT_ENABLED_TOOLS: &[&str] = &["docker", "npm"];

#[derive(Serialize, Deserialize, Default, Clone)]
struct Settings {
    /// List of enabled tool names.  Tools not in this list are disabled.
    /// An empty list (on fresh install) means "all enabled".
    #[serde(default)]
    enabled_tools: Vec<String>,
}

fn settings_path() -> Option<std::path::PathBuf> {
    ProjectDirs::from("", "", "tsm").map(|d| d.config_dir().join("settings.toml"))
}

/// Load settings.  Returns default (empty = all enabled) when file doesn't exist.
fn load_settings() -> Settings {
    let path = match settings_path() {
        Some(p) if p.exists() => p,
        _ => return Settings::default(),
    };
    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return Settings::default(),
    };
    toml::from_str(&content).unwrap_or_default()
}

fn save_settings(settings: &Settings) -> Result<()> {
    let path = settings_path()
        .ok_or_else(|| anyhow::anyhow!("Could not determine settings directory"))?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create settings dir: {:?}", parent))?;
    }
    let toml_str = toml::to_string_pretty(settings)?;
    fs::write(&path, toml_str)
        .with_context(|| format!("Failed to write settings: {:?}", path))?;
    Ok(())
}

/// Is the given tool enabled?
/// - Fresh install (settings file doesn't exist or enabled_tools is empty):
///   only the DEFAULT_ENABLED_TOOLS are enabled; everything else is disabled.
/// - After the user saves via `tsm tools`, only the listed tools are enabled.
pub fn is_tool_enabled(tool: &str) -> bool {
    let settings = load_settings();
    if settings.enabled_tools.is_empty() {
        // No settings saved yet — use defaults
        DEFAULT_ENABLED_TOOLS.contains(&tool)
    } else {
        settings.enabled_tools.iter().any(|t| t == tool)
    }
}

/// Persist the user's enabled-tool selection.
pub fn save_enabled_tools(tools: &[String]) -> Result<()> {
    save_settings(&Settings {
        enabled_tools: tools.to_vec(),
    })
}
