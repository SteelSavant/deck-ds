use std::process::Command;

use anyhow::{Context, Result};
use nix::unistd::Pid;

#[derive(Debug)]
pub struct FlatpakStatus {
    pub app_id: String,
    pub pid: Pid,
    pub instance: u32,
}

pub fn check_running_flatpaks() -> Result<Vec<FlatpakStatus>> {
    let output = Command::new("flatpak").arg("ps").output()?;
    if output.status.success() {
        let status = String::from_utf8_lossy(&output.stdout)
            .lines()
            // .skip(1)
            .map(|v| v.split_ascii_whitespace())
            .map(|mut v| {
                let status = FlatpakStatus {
                    instance: v
                        .next()
                        .with_context(|| "instance number expected")?
                        .parse()?,
                    pid: Pid::from_raw(v.next().with_context(|| "pid expected")?.parse()?),
                    app_id: v
                        .next()
                        .with_context(|| "expected flatpak app id")?
                        .to_string(),
                };
                Ok(status)
            })
            .collect::<Result<_>>()?;
        Ok(status)
    } else {
        Err(anyhow::anyhow!(
            String::from_utf8_lossy(&output.stderr).to_string()
        ))
    }
}

pub struct FlatpakInfo {
    pub name: String,
    pub app_id: String,
}

pub fn list_installed_flatpaks() -> Result<Vec<FlatpakInfo>> {
    let output = Command::new("flatpak").args(["list", "--app"]).output()?;
    if output.status.success() {
        let status = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|v| v.split_ascii_whitespace())
            .map(|mut v| {
                let status = FlatpakInfo {
                    name: v.next().with_context(|| "name expected")?.parse()?,
                    app_id: v
                        .next()
                        .with_context(|| "expected flatpak app id")?
                        .to_string(),
                };
                Ok(status)
            })
            .collect::<Result<_>>()?;
        Ok(status)
    } else {
        Err(anyhow::anyhow!(
            String::from_utf8_lossy(&output.stderr).to_string()
        ))
    }
}
