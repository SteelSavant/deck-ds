use std::{path::Path, process::Command};

use egui::TextBuffer;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use anyhow::Result;

const SYSTEM_DEVICES: [&str; 2] = ["filter-chain-source", "output.virtual-source"];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AudioDeviceInfo {
    pub name: String,
    pub description: Option<String>,
    pub channels: Option<u8>,
}

impl AudioDeviceInfo {
    pub fn from_name(name: String) -> Self {
        Self {
            name,
            description: None,
            channels: None,
        }
    }
}

pub fn get_audio_sinks() -> Vec<AudioDeviceInfo> {
    get_audio("sinks")
        .inspect_err(|err| log::error!("failed to get audio sinks: {err}"))
        .unwrap_or_default()
}

pub fn get_audio_sources() -> Vec<AudioDeviceInfo> {
    get_audio("sources")
        .inspect_err(|err| log::error!("failed to get audio sinks: {err}"))
        .unwrap_or_default()
}

fn get_audio(audio_type: &str) -> Result<Vec<AudioDeviceInfo>> {
    let mut cmd = Command::new("pactl");

    cmd.args(["list", audio_type]);

    let user = usdpl_back::api::decky::user();

    // TODO::don't hardcode this

    // if let Ok(user) = user {
    let runtime_dir = Path::new("/run").join("user").join("1000");
    log::debug!("Setting XDG_RUNTIME_DIR to {:?}", runtime_dir);
    cmd.env("XDG_RUNTIME_DIR", runtime_dir);
    // }

    let output = cmd.output()?;

    if output.status.success() {
        let out = String::from_utf8_lossy(&output.stdout);

        Ok(parse_pactl_list(&out)
            .into_iter()
            .filter(|v| {
                !v.name.ends_with(".monitor") && !SYSTEM_DEVICES.iter().any(|sd| *sd == &v.name)
            })
            .collect())
    } else {
        Err(anyhow::anyhow!(
            String::from_utf8_lossy(&output.stderr).to_string()
        ))
    }
}

/// Parse the pactl output. Badly.
fn parse_pactl_list(output: &str) -> Vec<AudioDeviceInfo> {
    let mut info = Vec::new();

    let mut name = String::new();
    let mut description = None;

    for line in output.lines() {
        let trimmed = line.trim_start();
        log::debug!("pactl line: {trimmed}");
        if let Some(captures) = trimmed.strip_prefix("Name: ") {
            log::debug!("pactl got name: {captures}");
            name = captures.trim().to_string();
        } else if let Some(captures) = trimmed.strip_prefix("Description: ") {
            log::debug!("pactl got description: {captures}");
            description = Some(captures.trim().to_string());
        } else if let Some(captures) = trimmed.strip_prefix("Sample Specification: ") {
            log::debug!(
                "pactl got spec: {:?}",
                captures.split_whitespace().collect::<Vec<_>>()
            );
            if let Some(channels_str) = captures.split_whitespace().nth(1) {
                if let Ok(channelsval) = channels_str.trim_end_matches("ch").parse::<u8>() {
                    log::debug!("pactl got channels: {channelsval:?}");

                    info.push(AudioDeviceInfo {
                        description: description.or(Some(name.clone())),
                        name: name.take(),
                        channels: Some(channelsval),
                    });
                    description = None;
                }
            }
        }
    }

    info
}
