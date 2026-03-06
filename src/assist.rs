//! AI-powered command error analysis and fix suggestions.

use crate::profile;
use anyhow::{bail, Context};
use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Debug, Parser, Clone)]
pub struct AssistCommand {
    /// The failed command string
    #[arg(long)]
    command: String,

    /// The exit code of the failed command
    #[arg(long)]
    exit_code: i32,

    /// Additional stderr output (optional)
    #[arg(long)]
    stderr: Option<String>,
}

#[derive(Deserialize)]
struct AssistantConfig {
    enabled: Option<bool>,
    api_key: Option<String>,
    model: Option<String>,
    base_url: Option<String>,
    custom_headers: Option<Vec<String>>,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: Message,
}

const DEFAULT_MODEL: &str = "DeepSeek-V3.2";
const DEFAULT_BASE_URL: &str = "https://api.vivgrid.com/v1";

impl AssistCommand {
    pub fn run(&self) -> anyhow::Result<()> {
        let config = load_config()?;

        if config.enabled == Some(false) {
            return Ok(());
        }

        let api_key = match &config.api_key {
            Some(k) if !k.is_empty() => k.clone(),
            _ => {
                eprintln!("\x1b[33m[kaku] No API key configured. Run `kaku ai` to set up.\x1b[0m");
                return Ok(());
            }
        };

        let model = config
            .model
            .unwrap_or_else(|| DEFAULT_MODEL.to_string());
        let base_url = config
            .base_url
            .unwrap_or_else(|| DEFAULT_BASE_URL.to_string());

        let system_prompt = format!(
            "You are a PowerShell command assistant. A command failed and you need to analyze why and suggest a fix.\n\
             Respond with ONLY the corrected command on the first line, then a blank line, then a brief explanation.\n\
             If you cannot determine a fix, say so briefly.\n\
             OS: Windows\n\
             Shell: PowerShell"
        );

        let user_msg = format!(
            "Failed command: {}\nExit code: {}\n{}",
            self.command,
            self.exit_code,
            self.stderr
                .as_deref()
                .map(|s| format!("stderr:\n{s}"))
                .unwrap_or_default()
        );

        let rt = tokio::runtime::Runtime::new()?;
        let result = rt.block_on(call_api(
            &base_url,
            &api_key,
            &model,
            &system_prompt,
            &user_msg,
            config.custom_headers.as_deref(),
        ))?;

        // Parse: first line = suggested command, rest = explanation
        let mut lines = result.lines();
        let suggestion = lines.next().unwrap_or("").trim();

        if !suggestion.is_empty() {
            // Save suggestion to temp file for Ctrl+Shift+E
            let temp = std::env::temp_dir().join("kaku_last_suggestion.txt");
            let _ = std::fs::write(&temp, suggestion);

            println!("\x1b[1;35m[kaku]\x1b[0m Suggested fix:");
            println!("  \x1b[1;32m{suggestion}\x1b[0m");

            let explanation: String = lines.collect::<Vec<_>>().join("\n");
            let explanation = explanation.trim();
            if !explanation.is_empty() {
                println!("  \x1b[90m{explanation}\x1b[0m");
            }
            println!("  \x1b[90mPress Ctrl+Shift+E to apply\x1b[0m");
        }

        Ok(())
    }
}

fn load_config() -> anyhow::Result<AssistantConfig> {
    let path = profile::assistant_toml_path();
    if !path.exists() {
        return Ok(AssistantConfig {
            enabled: Some(true),
            api_key: None,
            model: Some(DEFAULT_MODEL.to_string()),
            base_url: Some(DEFAULT_BASE_URL.to_string()),
            custom_headers: None,
        });
    }
    let raw = std::fs::read_to_string(&path)
        .with_context(|| format!("read {}", path.display()))?;
    let config: AssistantConfig = toml::from_str(&raw)
        .with_context(|| format!("parse {}", path.display()))?;
    Ok(config)
}

async fn call_api(
    base_url: &str,
    api_key: &str,
    model: &str,
    system_prompt: &str,
    user_msg: &str,
    custom_headers: Option<&[String]>,
) -> anyhow::Result<String> {
    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));

    let mut builder = reqwest::Client::new()
        .post(&url)
        .header("Authorization", format!("Bearer {api_key}"))
        .header("Content-Type", "application/json");

    if let Some(headers) = custom_headers {
        for h in headers {
            if let Some((name, value)) = h.split_once(':') {
                let name = name.trim();
                let lower = name.to_ascii_lowercase();
                if lower == "authorization" || lower == "content-type" {
                    continue;
                }
                builder = builder.header(name, value.trim());
            }
        }
    }

    let body = ChatRequest {
        model: model.to_string(),
        messages: vec![
            Message {
                role: "system".to_string(),
                content: system_prompt.to_string(),
            },
            Message {
                role: "user".to_string(),
                content: user_msg.to_string(),
            },
        ],
        max_tokens: 512,
        temperature: 0.3,
    };

    let resp = builder
        .json(&body)
        .send()
        .await
        .context("send request to AI API")?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        bail!("API returned {status}: {text}");
    }

    let chat: ChatResponse = resp.json().await.context("parse API response")?;
    let content = chat
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .unwrap_or_default();

    Ok(content)
}
