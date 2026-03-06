//! Initialize PowerShell profile with kaku integration.

use crate::profile;
use anyhow::{bail, Context};
use clap::Parser;
use std::fs;
use std::io::{self, IsTerminal, Write};

#[derive(Debug, Parser, Clone, Default)]
pub struct InitCommand {
    /// Only update existing integration, skip interactive prompts
    #[arg(long)]
    pub update_only: bool,
}

impl InitCommand {
    pub fn run(&self) -> anyhow::Result<()> {
        // Ensure config dir exists
        let config_dir = profile::config_dir();
        fs::create_dir_all(&config_dir)
            .with_context(|| format!("create {}", config_dir.display()))?;

        // Ensure assistant.toml exists
        let assistant_path = profile::assistant_toml_path();
        if !assistant_path.exists() {
            fs::write(&assistant_path, default_assistant_toml())
                .with_context(|| format!("write {}", assistant_path.display()))?;
            println!("  Created {}", assistant_path.display());
        }

        // Ensure kaku.toml exists
        let config_path = profile::config_toml_path();
        if !config_path.exists() {
            fs::write(&config_path, default_config_toml())
                .with_context(|| format!("write {}", config_path.display()))?;
            println!("  Created {}", config_path.display());
        }

        // Inject into PowerShell profile
        let profile_path = match profile::powershell_profile_path() {
            Some(p) => p,
            None => bail!("Cannot determine PowerShell profile path"),
        };

        if !self.update_only && io::stdin().is_terminal() {
            println!();
            println!("Kaku will add integration to:");
            println!("  {}", profile_path.display());
            print!("Continue? [Y/n] ");
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let answer = input.trim().to_ascii_lowercase();
            if !answer.is_empty() && answer != "y" && answer != "yes" {
                println!("Aborted.");
                return Ok(());
            }
        }

        // Resolve kaku executable path
        let kaku_exe = std::env::current_exe()
            .unwrap_or_else(|_| "kaku".into())
            .display()
            .to_string()
            .replace('\\', "\\\\");

        let block = profile::profile_block(&kaku_exe);

        // Read existing profile or create new
        if let Some(parent) = profile_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let existing = fs::read_to_string(&profile_path).unwrap_or_default();

        let new_content = if existing.contains(profile::PROFILE_MARKER) {
            // Replace existing block
            replace_managed_block(&existing, &block)
        } else {
            // Append
            if existing.is_empty() {
                block
            } else {
                format!("{existing}\n{block}")
            }
        };

        fs::write(&profile_path, &new_content)
            .with_context(|| format!("write {}", profile_path.display()))?;

        println!("\x1b[32m✓\x1b[0m PowerShell profile updated: {}", profile_path.display());
        println!("  Restart PowerShell or run: \x1b[1m. $PROFILE\x1b[0m");

        if !self.update_only {
            println!();
            suggest_optional_tools();
        }

        Ok(())
    }
}

fn replace_managed_block(content: &str, new_block: &str) -> String {
    let mut result = String::new();
    let mut inside_block = false;

    for line in content.lines() {
        if line.trim() == profile::PROFILE_MARKER {
            inside_block = true;
            continue;
        }
        if line.trim() == profile::PROFILE_MARKER_END {
            inside_block = false;
            continue;
        }
        if !inside_block {
            result.push_str(line);
            result.push('\n');
        }
    }

    // Trim trailing newlines then append
    let trimmed = result.trim_end_matches('\n');
    if trimmed.is_empty() {
        new_block.to_string()
    } else {
        format!("{trimmed}\n\n{new_block}")
    }
}

pub fn default_assistant_toml() -> &'static str {
    r#"# Kaku Assistant configuration
#
# enabled: true enables command analysis suggestions
# api_key: provider API key, example: "sk-xxxx"
# model: model id, example: "DeepSeek-V3.2"
# base_url: chat-completions API root URL

enabled = true
# api_key = "<your_api_key>"
model = "DeepSeek-V3.2"
base_url = "https://api.vivgrid.com/v1"
# custom_headers = ["X-Customer-ID: your-id"]
"#
}

pub fn default_config_toml() -> &'static str {
    r#"# Kaku Shell configuration

[tools]
# Uncomment and configure the tools you use:
# claude_code = true
# codex = true
# gemini_cli = true
# copilot_cli = true
# openclaw = true
"#
}

fn suggest_optional_tools() {
    println!("Optional CLI tools for a better experience:");
    println!("  \x1b[1mwinget install Starship.Starship\x1b[0m      # prompt");
    println!("  \x1b[1mwinget install dandavison.delta\x1b[0m       # git diff");
    println!("  \x1b[1mwinget install jesseduffield.lazygit\x1b[0m  # git TUI");
    println!("  \x1b[1mwinget install sxyazi.yazi\x1b[0m            # file manager");
    println!("  \x1b[1mwinget install ajeetdsouza.zoxide\x1b[0m     # smart cd");
}
