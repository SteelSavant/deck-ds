use std::{cmp::Ordering, path::Path};

use anyhow::Result;
use edid::EDID;
use itertools::Itertools;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DisplayInfo {
    pub model: String,
    pub serial: String,
    pub display_modes: Vec<DisplayMode>,
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DisplayMode {
    width: u16,
    height: u16,
    refresh: Option<f32>, // can't fetch it now, but I'd like to in the future if possible
}

impl Eq for DisplayMode {}

impl PartialOrd for DisplayMode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DisplayMode {
    fn cmp(&self, other: &Self) -> Ordering {
        let area = self.width * self.height;
        let other_area = other.width * other.height;

        if area < other_area {
            Ordering::Less
        } else if area > other_area {
            Ordering::Greater
        } else if self.refresh < other.refresh {
            Ordering::Less
        } else if self.refresh > other.refresh {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}

/// Gets raw display info direct from the system,
/// without going through X. Primarily for displaying
/// available monitors and modes in GameMode.
///
/// Returns an optional vec of [DisplayInfo], ordered
/// by greatest to least [DisplayMode].
pub fn get_display_info() -> Result<Vec<DisplayInfo>> {
    let device_dir = "/sys/class/drm";
    let mut info = Path::new(device_dir)
        .read_dir()?
        .into_iter()
        .filter_map(|v| {
            v.inspect_err(|err| log::warn!("failed to read dir entry: {:#?}", err))
                .ok()
        })
        .filter(|v| v.path().is_dir())
        .filter_map(|dir| {
            let status_path = dir.path().join("status");
            if status_path.is_file()
                && std::fs::read_to_string(status_path)
                    .unwrap_or_default()
                    .starts_with("connected")
            {
                return Some(dir.path());
            }

            None
        })
        .map(|dir| {
            let modes = parse_modes(dir.join("modes")).unwrap_or_default();

            let mut info = DisplayInfo {
                model: "Unknown".to_string(),
                serial: "Unknown".to_string(),
                display_modes: modes,
            };
            let edid = parse_edid(dir.join("edid"));
            if let Ok(edid) = edid {
                for d in edid.descriptors {
                    match d {
                        edid::Descriptor::SerialNumber(s) => info.serial = s,
                        edid::Descriptor::ProductName(p) => info.model = p,
                        _ => (),
                    }
                }
            }

            info
        })
        .collect_vec();

    info.sort_by(|a, b| b.display_modes.cmp(&a.display_modes));

    Ok(info)
}

fn parse_modes<P: AsRef<Path>>(file: P) -> Option<Vec<DisplayMode>> {
    let modes = std::fs::read_to_string(file).ok()?;

    let mut modes = modes
        .split_terminator('\n')
        .filter(|v| !v.trim().is_empty())
        .map(|v| {
            let mut res = v.split_terminator('x');
            let width = res
                .next()
                .expect("expected width when parsing mode")
                .parse()
                .unwrap();
            let height = res
                .next()
                .expect("expected height when parsing mode")
                .parse()
                .unwrap();

            DisplayMode {
                width,
                height,
                refresh: None,
            }
        })
        .collect::<Vec<_>>();

    modes.dedup();

    Some(modes)
}

fn parse_edid<P: AsRef<Path>>(file: P) -> Result<EDID> {
    let bytes = std::fs::read(file)?;
    let res = edid::parse(&bytes);

    if res.is_done() {
        Ok(res.unwrap().1)
    } else if res.is_err() {
        Err(anyhow::format_err!(
            "Failed to parse EDID: {:#?}",
            res.unwrap_err()
        ))
    } else {
        anyhow::bail!("EDID incomplete: {:#?}", res.unwrap_inc())
    }
}
