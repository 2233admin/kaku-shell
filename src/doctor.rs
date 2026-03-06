//! Environment diagnostics for PowerShell + AI tool health.

use crate::profile;
use std::io::Write;
use std::process::Command;

struct Check {
    name: &'static str,
    status: Status,
    detail: String,
    fix: Option<String>,
}

#[derive(Clone, Copy)]
enum Status {
    Ok,
    Warn,
    Fail,
    Info,
}

impl Status {
    fn icon(self) -> &'static str {
        match self {
            Self::Ok => "\x1b[32m✓\x1b[0m",
            Self::Warn => "\x1b[33m!\x1b[0m",
            Self::Fail => "\x1b[31mx\x1b[0m",
            Self::Info => "\x1b[36mi\x1b[0m",
        }
    }
}

pub fn run() -> anyhow::Result<()> {
    println!("\x1b[1mKaku Doctor\x1b[0m\n");

    let checks = vec![
        check_powershell(),
        check_profile_integration(),
        check_assistant_config(),
        check_tool("starship", "starship --version", "winget install Starship.Starship"),
        check_tool("delta", "delta --version", "winget install dandavison.delta"),
        check_tool("lazygit", "lazygit --version", "winget install jesseduffield.lazygit"),
        check_tool("yazi", "yazi --version", "winget install sxyazi.yazi"),
        check_tool("zoxide", "zoxide --version", "winget install ajeetdsouza.zoxide"),
    ];

    let mut ok = 0;
    let mut warn = 0;
    let mut fail = 0;

    for c in &checks {
        let icon = c.status.icon();
        println!("  {icon} {}: {}", c.name, c.detail);
        if let Some(fix) = &c.fix {
            println!("    \x1b[90mFix: {fix}\x1b[0m");
        }
        match c.status {
            Status::Ok => ok += 1,
            Status::Warn => warn += 1,
            Status::Fail => fail += 1,
            Status::Info => {}
        }
    }

    println!();
    println!("  {ok} ok  {warn} warn  {fail} fail");
    std::io::stdout().flush()?;
    Ok(())
}

fn check_powershell() -> Check {
    let version = Command::new("pwsh")
        .args(["--version"])
        .output()
        .or_else(|_| Command::new("powershell").args(["--version"]).output());

    match version {
        Ok(out) if out.status.success() => {
            let v = String::from_utf8_lossy(&out.stdout).trim().to_string();
            Check {
                name: "PowerShell",
                status: if v.contains("7.") { Status::Ok } else { Status::Warn },
                detail: v.clone(),
                fix: if v.contains("7.") {
                    None
                } else {
                    Some("winget install Microsoft.PowerShell".into())
                },
            }
        }
        _ => Check {
            name: "PowerShell",
            status: Status::Fail,
            detail: "PowerShell not found".into(),
            fix: Some("winget install Microsoft.PowerShell".into()),
        },
    }
}

fn check_profile_integration() -> Check {
    let path = match profile::powershell_profile_path() {
        Some(p) => p,
        None => {
            return Check {
                name: "Profile Integration",
                status: Status::Warn,
                detail: "Cannot determine PowerShell profile path".into(),
                fix: Some("Run `kaku init`".into()),
            }
        }
    };

    if !path.exists() {
        return Check {
            name: "Profile Integration",
            status: Status::Warn,
            detail: format!("Profile not found: {}", path.display()),
            fix: Some("Run `kaku init`".into()),
        };
    }

    let content = std::fs::read_to_string(&path).unwrap_or_default();
    if content.contains(profile::PROFILE_MARKER) {
        Check {
            name: "Profile Integration",
            status: Status::Ok,
            detail: format!("Kaku block found in {}", path.display()),
            fix: None,
        }
    } else {
        Check {
            name: "Profile Integration",
            status: Status::Warn,
            detail: format!("No kaku block in {}", path.display()),
            fix: Some("Run `kaku init`".into()),
        }
    }
}

fn check_assistant_config() -> Check {
    let path = profile::assistant_toml_path();
    if !path.exists() {
        return Check {
            name: "AI Assistant Config",
            status: Status::Warn,
            detail: format!("Missing {}", path.display()),
            fix: Some("Run `kaku ai` to configure".into()),
        };
    }

    let raw = std::fs::read_to_string(&path).unwrap_or_default();
    let has_key = raw.lines().any(|l| {
        let t = l.trim();
        !t.starts_with('#') && t.starts_with("api_key") && t.contains('=')
    });

    if has_key {
        Check {
            name: "AI Assistant Config",
            status: Status::Ok,
            detail: "API key configured".into(),
            fix: None,
        }
    } else {
        Check {
            name: "AI Assistant Config",
            status: Status::Warn,
            detail: "API key not set".into(),
            fix: Some("Run `kaku ai` to set API key".into()),
        }
    }
}

fn check_tool(name: &'static str, version_cmd: &str, install_cmd: &str) -> Check {
    let parts: Vec<&str> = version_cmd.split_whitespace().collect();
    let result = Command::new(parts[0]).args(&parts[1..]).output();

    match result {
        Ok(out) if out.status.success() => {
            let v = String::from_utf8_lossy(&out.stdout)
                .lines()
                .next()
                .unwrap_or("")
                .trim()
                .to_string();
            Check {
                name,
                status: Status::Ok,
                detail: if v.is_empty() { "installed".into() } else { v },
                fix: None,
            }
        }
        _ => Check {
            name,
            status: Status::Info,
            detail: "not installed (optional)".into(),
            fix: Some(install_cmd.into()),
        },
    }
}
