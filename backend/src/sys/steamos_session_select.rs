use anyhow::{anyhow, Result};
use std::process::Command;

#[derive(Debug, Clone, Copy)]
pub enum Session {
    Gamescope,
    Plasma,
}

pub fn steamos_session_select(session: Session) -> Result<()> {
    let s = match session {
        Session::Gamescope => "gamescope",
        Session::Plasma => "plasma",
    };

    Command::new("steamos-session-select")
        .arg(s)
        .env("SHELL", "/bin/sh") // At some point during development, SteamOS lost the ability to set this correctly...
        .output()
        .map(|output| {
            if output.status.success() {
                Ok(())
            } else {
                Err(anyhow!(
                    "steamos-session-select failed with error {:?}, code {:?}",
                    String::from_utf8_lossy(&output.stderr),
                    output.status.code()
                ))
            }
        })?
}
