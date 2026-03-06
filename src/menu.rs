//! Interactive main menu when `kaku` is run without subcommand.

use crate::SubCommand;
use anyhow::Context;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::io::Write;

const GREEN: &str = "\x1b[0;32m";
const PURPLE_BOLD: &str = "\x1b[1;35m";
const GRAY: &str = "\x1b[0;90m";
const RESET: &str = "\x1b[0m";

struct MenuItem {
    key: &'static str,
    desc: &'static str,
    cmd: fn() -> SubCommand,
}

const MENU: &[MenuItem] = &[
    MenuItem {
        key: "ai",
        desc: "Manage AI assistant and tool settings",
        cmd: || SubCommand::Ai,
    },
    MenuItem {
        key: "config",
        desc: "Open ~/.config/kaku/kaku.toml",
        cmd: || SubCommand::Config,
    },
    MenuItem {
        key: "init",
        desc: "Initialize PowerShell profile integration",
        cmd: || SubCommand::Init(Default::default()),
    },
    MenuItem {
        key: "doctor",
        desc: "Run diagnostics for shell and tool health",
        cmd: || SubCommand::Doctor,
    },
    MenuItem {
        key: "reset",
        desc: "Remove kaku PowerShell profile integration",
        cmd: || SubCommand::Reset,
    },
];

pub fn select_main_menu() -> anyhow::Result<Option<SubCommand>> {
    enable_raw_mode().context("enable raw mode")?;
    let _guard = RawGuard;

    let mut selected = 0usize;
    render(selected)?;

    loop {
        match event::read().context("read input")? {
            Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    if selected > 0 {
                        selected -= 1;
                        render(selected)?;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if selected + 1 < MENU.len() {
                        selected += 1;
                        render(selected)?;
                    }
                }
                KeyCode::Enter => return Ok(Some((MENU[selected].cmd)())),
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    return Ok(None)
                }
                KeyCode::Char(c @ '1'..='5') if no_modifiers(key.modifiers) => {
                    let idx = (c as usize) - ('1' as usize);
                    return Ok(Some((MENU[idx].cmd)()));
                }
                KeyCode::Char('q') | KeyCode::Esc => return Ok(None),
                _ => {}
            },
            _ => {}
        }
    }
}

fn no_modifiers(m: KeyModifiers) -> bool {
    !m.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SUPER)
}

fn render(selected: usize) -> anyhow::Result<()> {
    use crossterm::cursor::MoveTo;
    use crossterm::queue;
    use crossterm::terminal::{Clear, ClearType};

    let mut out = std::io::stdout();
    queue!(out, MoveTo(0, 0), Clear(ClearType::All))?;

    let mut buf = String::new();
    buf.push_str("\r\n");
    buf.push_str(&format!("{GREEN}  _  __      _          {RESET}\r\n"));
    buf.push_str(&format!("{GREEN} | |/ /     | |         {RESET}\r\n"));
    buf.push_str(&format!("{GREEN} | ' / __ _ | | __ _   _ {RESET}\r\n"));
    buf.push_str(&format!("{GREEN} |  < / _` || |/ /| | | |{RESET}\r\n"));
    buf.push_str(&format!(
        "{GREEN} | . \\ (_| ||   < | |_| |{RESET}  {PURPLE_BOLD}PowerShell AI Terminal{RESET}\r\n"
    ));
    buf.push_str(&format!(
        "{GREEN} |_|\\_\\__,_||_|\\_\\ \\__,_|{RESET}  {GREEN}github.com/2233admin/kaku-shell{RESET}\r\n"
    ));
    buf.push_str("\r\n");

    for (i, item) in MENU.iter().enumerate() {
        let n = i + 1;
        if i == selected {
            buf.push_str(&format!(
                "{PURPLE_BOLD}> {n}. {:<7}     {}{RESET}\r\n",
                item.key, item.desc
            ));
        } else {
            buf.push_str(&format!("  {n}. {:<7}     {}\r\n", item.key, item.desc));
        }
    }
    buf.push_str("\r\n");
    buf.push_str(&format!(
        "  {GRAY}arrows | Enter | 1-5 | Q quit{RESET}\r\n"
    ));

    out.write_all(buf.as_bytes())?;
    out.flush()?;
    Ok(())
}

struct RawGuard;
impl Drop for RawGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
    }
}
