//! Open kaku configuration file.

use crate::profile;
use anyhow::Context;
use std::fs;

pub fn run() -> anyhow::Result<()> {
    let path = profile::config_toml_path();

    let config_dir = profile::config_dir();
    fs::create_dir_all(&config_dir)?;

    if !path.exists() {
        fs::write(&path, crate::init::default_config_toml())
            .with_context(|| format!("write {}", path.display()))?;
        println!("Created {}", path.display());
    }

    let editor = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| "notepad".to_string());

    println!("Opening {} with {editor}", path.display());
    std::process::Command::new(&editor)
        .arg(&path)
        .status()
        .with_context(|| format!("launch {editor}"))?;

    Ok(())
}
