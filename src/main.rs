mod config;
mod error;
mod sources;
mod traits;
mod types;
mod utils;

use anyhow::{bail, Result};
use clap::{Parser, Subcommand};
use sources::get_manager;
use types::Mirror;

#[derive(Parser)]
#[command(name = "tsm")]
#[command(
    about = "A fast, interactive mirror manager for dev tools",
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List current mirror configuration (e.g., tsm ls [pip])
    Ls {
        /// The tool name (pip, docker, etc.). If omitted, shows all enabled tools.
        name: Option<String>,
    },
    /// Benchmark mirrors (e.g., tsm test pip)
    Test {
        /// The tool name
        name: String,
    },
    /// Apply a new mirror (e.g., tsm use pip --fastest)
    Use {
        /// The tool name
        name: String,

        /// Mirror alias (e.g., Aliyun)
        #[arg(required_unless_present = "fastest")]
        source: Option<String>,

        /// Auto-select the fastest mirror
        #[arg(long, short)]
        fastest: bool,
    },
    /// Restore the configuration to the previous backup or default
    Restore {
        /// The tool name
        name: String,
    },
    /// Manage custom mirror sources
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
    /// Enable or disable tools interactively
    Tools,
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// Add a custom mirror source (e.g., tsm config add pip https://example.com/pypi/simple/)
    Add {
        /// The tool name (pip, npm, docker, etc.)
        tool: String,
        /// The mirror URL to add
        url: String,
    },
    /// Remove mirror sources interactively (e.g., tsm config rm pip)
    Rm {
        /// The tool name
        tool: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // On first run, seed the user config with built-in defaults.
    // On subsequent runs the file already exists, so this is a no-op.
    config::ensure_user_config_initialized()?;

    match cli.command {
        Commands::Ls { name } => handle_ls(name).await?,
        Commands::Test { name } => handle_test(&name).await?,
        Commands::Use {
            name,
            source,
            fastest,
        } => handle_use(&name, source, fastest).await?,
        Commands::Restore { name } => handle_restore(&name).await?,
        Commands::Config { command } => match command {
            ConfigCommands::Add { tool, url } => handle_config_add(&tool, &url).await?,
            ConfigCommands::Rm { tool } => handle_config_rm(&tool).await?,
        },
        Commands::Tools => handle_tools().await?,
    }

    Ok(())
}

// ── Handlers ──────────────────────────────────────────────────────────────

async fn ensure_enabled(tool: &str) -> Result<()> {
    if !config::is_tool_enabled(tool) {
        println!("{} not enabled", tool);
        // Return Ok to avoid a noisy error — the message is enough
        std::process::exit(0);
    }
    Ok(())
}

async fn handle_ls(name: Option<String>) -> Result<()> {
    if let Some(ref n) = name {
        ensure_enabled(n).await?;
    }

    let tools = match name {
        Some(n) => vec![n],
        None => sources::SUPPORTED_TOOLS
            .iter()
            .filter(|t| config::is_tool_enabled(t))
            .map(|&s| s.to_string())
            .collect(),
    };

    println!("{}", "-".repeat(70));
    println!("{:<10} {:<40} Status", "Tool", "Current Source URL");
    println!("{}", "-".repeat(70));

    for tool_name in tools {
        let manager = match get_manager(&tool_name) {
            Ok(m) => m,
            Err(_) => continue,
        };

        let current_url_res = manager.current_url().await;
        let current_url = current_url_res.unwrap_or_default();
        let candidates = manager.list_candidates();

        let (url_display, status_display) = match current_url {
            Some(url) => {
                let known_name = candidates
                    .iter()
                    .find(|m| m.url.trim_end_matches('/') == url.trim_end_matches('/'))
                    .map(|m| m.name.clone())
                    .unwrap_or_else(|| "Custom".to_string());
                (url, format!("[{}]", known_name))
            }
            None => ("Default".to_string(), "[Official/Default]".to_string()),
        };

        let mut url_short = url_display.clone();
        if url_short.len() > 38 {
            url_short = format!("{}...", &url_short[..35]);
        }

        println!(
            "{:<10} {:<40} {}",
            manager.name(),
            url_short,
            status_display
        );
    }
    println!("{}", "-".repeat(70));

    Ok(())
}

async fn handle_test(name: &str) -> Result<()> {
    ensure_enabled(name).await?;

    let manager = get_manager(name)?;
    let mut candidates = manager.list_candidates();

    let mut current_url_opt = manager.current_url().await.ok().flatten();

    if current_url_opt.is_none() {
        if let Some(official) = candidates
            .iter()
            .find(|m| m.name.eq_ignore_ascii_case("Official"))
        {
            current_url_opt = Some(official.url.clone());
        }
    }

    if let Some(ref current_url) = current_url_opt {
        let is_known = candidates.iter().any(|m| {
            m.url == *current_url
                || m.url.trim_end_matches('/') == current_url.trim_end_matches('/')
        });

        if !is_known {
            candidates.push(Mirror::new("Current", current_url));
        }
    }

    let results = utils::benchmark_mirrors(candidates).await;

    println!();
    println!();

    println!("{:<4} {:<10} {:<12} URL", "RANK", "LATENCY", "NAME");
    println!("{}", "-".repeat(60));

    for (i, res) in results.iter().enumerate() {
        let latency_str = if res.latency_ms == u64::MAX {
            "Timeout".to_string()
        } else {
            format!("{}ms", res.latency_ms)
        };

        println!(
            "{:<4} {:<10} {:<12} {}",
            i + 1,
            latency_str,
            res.mirror.name,
            res.mirror.url
        );
    }

    if let Some(best) = results.first() {
        if best.latency_ms < u64::MAX {
            println!("{}", "-".repeat(60));

            let mut speedup_msg = None;

            if let Some(ref current_url) = current_url_opt {
                let current_res = results.iter().find(|r| {
                    r.mirror.url == *current_url
                        || r.mirror.url.trim_end_matches('/')
                            == current_url.trim_end_matches('/')
                });

                if let Some(cur) = current_res {
                    if cur.latency_ms < u64::MAX {
                        if cur.latency_ms > best.latency_ms {
                            let speedup = cur.latency_ms as f64 / best.latency_ms as f64;
                            speedup_msg = Some(format!(
                                "Recommendation: '{}' is {:.1}x faster than your current source.",
                                best.mirror.name, speedup
                            ));
                        } else if cur.latency_ms == best.latency_ms {
                            speedup_msg = Some(format!(
                                "Recommendation: Your current source '{}' is already the fastest.",
                                cur.mirror.name
                            ));
                        }
                    } else {
                        speedup_msg = Some(format!(
                            "Recommendation: '{}' is significantly faster than your current source (Timeout).",
                            best.mirror.name
                        ));
                    }
                }
            }

            if let Some(msg) = speedup_msg {
                println!("{}", msg);
            } else {
                println!("Recommendation: '{}' is the fastest.", best.mirror.name);
            }

            println!("Run 'tsm use {} {}' to apply.", name, best.mirror.name);
        }
    }

    Ok(())
}

async fn handle_use(name: &str, source_name: Option<String>, fastest: bool) -> Result<()> {
    ensure_enabled(name).await?;

    let manager = get_manager(name)?;

    if manager.requires_sudo() {
        eprintln!(
            "Note: Modifying {} config usually requires sudo/root permissions.",
            name
        );
    }

    let target_mirror = if fastest {
        println!("Finding fastest mirror...");
        let results = utils::benchmark_mirrors(manager.list_candidates()).await;

        let valid_results: Vec<_> = results
            .into_iter()
            .filter(|r| r.latency_ms < u64::MAX)
            .collect();

        if valid_results.is_empty() {
            bail!("All mirrors timed out. Please check your network connection.");
        }

        let best = &valid_results[0];
        println!(
            "Fastest mirror is {} ({}ms)",
            best.mirror.name, best.latency_ms
        );
        best.mirror.clone()
    } else {
        let target_name = source_name.unwrap();
        let candidates = manager.list_candidates();

        match candidates
            .into_iter()
            .find(|m| m.name.eq_ignore_ascii_case(&target_name))
        {
            Some(m) => m,
            None => bail!(
                "Mirror '{}' not found. Use 'test' to see available list.",
                target_name
            ),
        }
    };

    println!("Backing up and applying {}...", target_mirror.name);
    manager.set_source(&target_mirror).await?;
    println!("Success! {} is now using {}.", name, target_mirror.name);

    Ok(())
}

async fn handle_restore(name: &str) -> Result<()> {
    ensure_enabled(name).await?;

    let manager = get_manager(name)?;

    if manager.requires_sudo() {
        eprintln!(
            "Note: Restoring {} config usually requires sudo/root permissions.",
            name
        );
    }

    println!("Restoring {} configuration...", name);
    manager.restore().await?;
    println!("Success! {} configuration restored.", name);

    Ok(())
}

// ── Config handlers ───────────────────────────────────────────────────────

async fn handle_config_add(tool: &str, url: &str) -> Result<()> {
    // Validate the tool is supported
    let _manager = get_manager(tool)?;

    // Parse URL to extract hostname for auto-generated name
    let name = utils::extract_hostname(url)?;

    // Check for duplicate URL (against merged candidates)
    let candidates = config::get_candidates(tool);
    let norm = utils::normalize_url(url);
    if candidates
        .iter()
        .any(|m| utils::normalize_url(&m.url) == norm)
    {
        bail!("Mirror with URL '{}' already exists for {}.", url, tool);
    }

    // Load user config, add, save
    let mut user_config = config::load_user_config()?;
    let mirrors = user_config.entry(tool.to_string()).or_default();
    mirrors.push(Mirror::new(&name, url));
    config::save_user_config(&user_config)?;

    println!("Added '{}' ({}) to {}.", name, url, tool);
    Ok(())
}

async fn handle_config_rm(tool: &str) -> Result<()> {
    // Validate the tool is supported
    let _manager = get_manager(tool)?;

    // Load user config mirrors for this tool
    let mut user_config = config::load_user_config()?;
    let mirrors = user_config.get(tool).cloned().unwrap_or_default();

    if mirrors.is_empty() {
        println!("No custom mirrors configured for {}.", tool);
        return Ok(());
    }

    // Build display items
    let items: Vec<String> = mirrors
        .iter()
        .map(|m| format!("{} — {}", m.name, m.url))
        .collect();

    let selections = dialoguer::MultiSelect::new()
        .with_prompt(format!(
            "Select mirrors to remove from {} (Space to toggle, Enter to confirm)",
            tool
        ))
        .items(&items)
        .interact()?;

    if selections.is_empty() {
        println!("No mirrors selected for removal.");
        return Ok(());
    }

    // Remove selected (iterate in reverse to maintain indices)
    let mut new_mirrors = mirrors;
    for &idx in selections.iter().rev() {
        let removed = &new_mirrors[idx];
        println!("Removed: {} ({})", removed.name, removed.url);
        new_mirrors.remove(idx);
    }

    // Update and save
    if new_mirrors.is_empty() {
        user_config.remove(tool);
    } else {
        user_config.insert(tool.to_string(), new_mirrors);
    }
    config::save_user_config(&user_config)?;

    Ok(())
}

// ── Tools handler ─────────────────────────────────────────────────────────

async fn handle_tools() -> Result<()> {
    let supported: Vec<&str> = sources::SUPPORTED_TOOLS.iter().copied().collect();

    // Build display items: ✓ for enabled, blank for disabled
    let items: Vec<String> = supported
        .iter()
        .map(|&t| {
            if config::is_tool_enabled(t) {
                format!("✓ {}", t)
            } else {
                format!("  {}", t)
            }
        })
        .collect();

    // Default selection: currently enabled tools
    let defaults: Vec<bool> = supported
        .iter()
        .map(|&t| config::is_tool_enabled(t))
        .collect();

    let selections = dialoguer::MultiSelect::new()
        .with_prompt("Toggle tools (Space to toggle, Enter when done)")
        .items(&items)
        .defaults(&defaults)
        .interact()?;

    // Selections are the indices the user left ON
    let enabled: Vec<String> = selections
        .iter()
        .map(|&i| supported[i].to_string())
        .collect();

    config::save_enabled_tools(&enabled)?;
    println!("Saved.");

    Ok(())
}
