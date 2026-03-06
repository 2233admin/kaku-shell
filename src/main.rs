use anyhow::Context;
use clap::Parser;
use std::io::IsTerminal;

mod ai_config;
mod assist;
mod config_cmd;
mod doctor;
mod init;
mod menu;
mod profile;
mod reset;
mod tui_core;

#[derive(Debug, Parser)]
#[command(
    name = "kaku",
    about = "Kaku Shell - Turn PowerShell into an AI coding terminal",
    version
)]
pub struct Opt {
    #[command(subcommand)]
    cmd: Option<SubCommand>,
}

#[derive(Debug, Parser, Clone)]
pub enum SubCommand {
    #[command(about = "Manage AI assistant and coding tools configuration")]
    Ai,

    #[command(about = "Analyze a failed command and suggest a fix")]
    Assist(assist::AssistCommand),

    #[command(about = "Open kaku configuration")]
    Config,

    #[command(about = "Diagnose shell environment and tool health")]
    Doctor,

    #[command(about = "Initialize PowerShell profile integration")]
    Init(init::InitCommand),

    #[command(about = "Reset PowerShell profile changes")]
    Reset,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {:#}", e);
        std::process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    let opts = Opt::parse();

    let cmd = if let Some(cmd) = opts.cmd {
        cmd
    } else if std::io::stdin().is_terminal() && std::io::stdout().is_terminal() {
        match menu::select_main_menu()? {
            Some(cmd) => cmd,
            None => return Ok(()),
        }
    } else {
        SubCommand::Doctor
    };

    match cmd {
        SubCommand::Ai => ai_config::run().context("ai config"),
        SubCommand::Assist(cmd) => cmd.run().context("assist"),
        SubCommand::Config => config_cmd::run().context("config"),
        SubCommand::Doctor => doctor::run().context("doctor"),
        SubCommand::Init(cmd) => cmd.run().context("init"),
        SubCommand::Reset => reset::run().context("reset"),
    }
}
