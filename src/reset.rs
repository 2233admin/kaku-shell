//! Reset: remove kaku integration from PowerShell profile.

use crate::profile;
use anyhow::Context;
use std::fs;
use std::io::{self, IsTerminal, Write};

pub fn run() -> anyhow::Result<()> {
    let profile_path = match profile::powershell_profile_path() {
        Some(p) => p,
        None => {
            println!("Cannot determine PowerShell profile path.");
            return Ok(());
        }
    };

    if !profile_path.exists() {
        println!("No profile found at {}. Nothing to reset.", profile_path.display());
        return Ok(());
    }

    let content = fs::read_to_string(&profile_path)
        .with_context(|| format!("read {}", profile_path.display()))?;

    if !content.contains(profile::PROFILE_MARKER) {
        println!("No kaku block found in {}. Nothing to reset.", profile_path.display());
        return Ok(());
    }

    if io::stdin().is_terminal() {
        print!("Remove kaku integration from {}? [y/N] ", profile_path.display());
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let answer = input.trim().to_ascii_lowercase();
        if answer != "y" && answer != "yes" {
            println!("Aborted.");
            return Ok(());
        }
    }

    let cleaned = remove_managed_block(&content);
    fs::write(&profile_path, &cleaned)
        .with_context(|| format!("write {}", profile_path.display()))?;

    println!("\x1b[32m✓\x1b[0m Removed kaku block from {}", profile_path.display());
    println!("  Restart PowerShell or run: \x1b[1m. $PROFILE\x1b[0m");
    Ok(())
}

fn remove_managed_block(content: &str) -> String {
    let mut result = String::new();
    let mut inside = false;

    for line in content.lines() {
        if line.trim() == profile::PROFILE_MARKER {
            inside = true;
            continue;
        }
        if line.trim() == profile::PROFILE_MARKER_END {
            inside = false;
            continue;
        }
        if !inside {
            result.push_str(line);
            result.push('\n');
        }
    }

    // Clean up excessive trailing newlines
    while result.ends_with("\n\n") {
        result.pop();
    }

    result
}
