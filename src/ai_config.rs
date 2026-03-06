//! AI tools configuration TUI.

use crate::profile;
use anyhow::Context;
use std::fs;
use std::io;

pub fn run() -> anyhow::Result<()> {
    let config_dir = profile::config_dir();
    fs::create_dir_all(&config_dir)?;

    let path = profile::assistant_toml_path();
    if !path.exists() {
        fs::write(&path, crate::init::default_assistant_toml())
            .with_context(|| format!("write {}", path.display()))?;
    }

    // For now, open the file in the default editor.
    // TODO: Full ratatui TUI for interactive editing.
    println!("\x1b[1;35m[kaku ai]\x1b[0m Assistant configuration\n");

    let raw = fs::read_to_string(&path)
        .with_context(|| format!("read {}", path.display()))?;

    // Display current config
    let enabled = extract_value(&raw, "enabled").unwrap_or("true");
    let model = extract_value(&raw, "model").unwrap_or("DeepSeek-V3.2");
    let base_url = extract_value(&raw, "base_url").unwrap_or("https://api.vivgrid.com/v1");
    let has_key = extract_value(&raw, "api_key").is_some();

    println!("  Enabled:  {enabled}");
    println!("  Model:    {model}");
    println!("  Base URL: {base_url}");
    println!(
        "  API Key:  {}",
        if has_key {
            "\x1b[32m configured \x1b[0m"
        } else {
            "\x1b[33m not set \x1b[0m"
        }
    );
    println!("\n  Config: {}\n", path.display());

    // Interactive: ask to open editor
    print!("Open in editor? [Y/n] ");
    io::Write::flush(&mut io::stdout())?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let answer = input.trim().to_ascii_lowercase();
    if answer.is_empty() || answer == "y" || answer == "yes" {
        open_in_editor(&path)?;
    }

    Ok(())
}

fn extract_value<'a>(content: &'a str, key: &str) -> Option<&'a str> {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = trimmed.split_once('=') {
            if k.trim() == key {
                return Some(v.trim().trim_matches('"'));
            }
        }
    }
    None
}

fn open_in_editor(path: &std::path::Path) -> anyhow::Result<()> {
    let editor = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| "notepad".to_string());

    std::process::Command::new(&editor)
        .arg(path)
        .status()
        .with_context(|| format!("launch {editor}"))?;

    Ok(())
}
