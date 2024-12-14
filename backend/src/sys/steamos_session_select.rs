use anyhow::Result;
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
                anyhow::bail!(
                    "steamos-session-select failed with error {:?}, code {:?}",
                    String::from_utf8_lossy(&output.stderr),
                    output.status.code()
                )
            }
        })?
}

pub fn check_session() -> Result<Session> {
    Command::new("w").output().map(|output| {
        if output.status.success() {
            let output = String::from_utf8_lossy(&output.stdout);
            if output.contains("gamescope-session") {
                Ok(Session::Gamescope)
            } else {
                Ok(Session::Plasma)
            }
        } else {
            anyhow::bail!(
                "w failed with error {:?}, code {:?}",
                String::from_utf8_lossy(&output.stderr),
                output.status.code()
            )
        }
    })?
}
