use std::{path::Path, process::Command};

use egui::TextBuffer;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use anyhow::{Context, Result};

const SYSTEM_DEVICES: [&str; 6] = [
    "filter-chain-source",
    "filter-chain-sink.monitor",
    "input.virtual-sink.monitor",
    "output.virtual-source",
    "alsa_output.pci-0000_04_00.5-platform-acp5x_mach.0.HiFi__hw_acp5x_1__sink.monitor",
    "alsa_output.pci-0000_04_00.5-platform-acp5x_mach.0.HiFi__hw_acp5x_0__sink.monitor",
];

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

    // if let Ok(user) = user {
    let runtime_dir = Path::new("/run").join("user").join("1000");
    log::debug!("Setting XDG_RUNTIME_DIR to {:?}", runtime_dir);
    cmd.env("XDG_RUNTIME_DIR", runtime_dir);
    // }

    let output = cmd.output()?;

    if output.status.success() {
        let out = String::from_utf8_lossy(&output.stdout);

        Ok(parse_pactl_list(&out))
    } else {
        Err(anyhow::anyhow!(
            String::from_utf8_lossy(&output.stderr).to_string()
        ))
    }
}

/// Parse the pactl output. Badly.
fn parse_pactl_list(output: &str) -> Vec<AudioDeviceInfo> {
    log::debug!("Got pactl output {output}");

    let mut info = Vec::new();

    let mut name = String::new();
    let mut description = None;

    // TODO::fix this

    for line in output.lines() {
        let trimmed = line.trim_start();
        if let Some(captures) = trimmed.strip_prefix("Name: ") {
            name = captures.trim().to_string();
        } else if let Some(captures) = trimmed.strip_prefix("Description: ") {
            description = Some(captures.trim().to_string());
        } else if let Some(captures) = trimmed.strip_prefix("Sample Specification: ") {
            if let Some(channels_str) = captures.split_whitespace().nth(1) {
                if let Ok(channelsval) = channels_str.parse::<u8>() {
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
