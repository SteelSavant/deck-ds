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

    Ok(Command::new("steamos-session-select")
        .arg(s)
        .status()
        .map(|status| {
            if status.success() {
                Ok(())
            } else {
                Err(anyhow!(
                    "steamos-session-select failed with error code {:?}",
                    status.code()
                ))
            }
        })??)
}
