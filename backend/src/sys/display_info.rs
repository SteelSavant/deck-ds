use std::{
    cmp::Ordering,
    path::{Path, PathBuf},
};

use anyhow::Result;
use edid::EDID;
use itertools::Itertools;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::settings_db::MonitorId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DisplayInfo {
    pub model: String,
    pub serial: String,
    pub display_modes: Vec<DisplayMode>,
    /// Path to the device folder on the system
    pub sys_path: PathBuf,
}

impl DisplayInfo {
    pub fn get_id(&self) -> MonitorId {
        MonitorId::from_display_info(self)
    }
}

impl PartialOrd for DisplayInfo {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DisplayInfo {
    fn cmp(&self, other: &Self) -> Ordering {
        let ord = self.display_modes.first().cmp(&other.display_modes.first());

        if ord != Ordering::Equal {
            return ord;
        }

        self.sys_path.cmp(&other.sys_path)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DisplayMode {
    pub width: u32,
    pub height: u32,
    // pub refresh: Option<f64>,
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
        // } else if self.refresh < other.refresh {
        //     Ordering::Less
        // } else if self.refresh > other.refresh {
        //     Ordering::Greater
        } else {
            self.width.cmp(&other.width)
        }
    }
}

/// Gets raw display info direct from the system,
/// without going through X.
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
            let mut modes = parse_modes(dir.join("modes")).unwrap_or_default();
            modes.sort_by(|a, b| b.cmp(a));

            let edid = parse_edid(dir.join("edid"));

            let mut info = DisplayInfo {
                model: MonitorId::UNKNOWN_STR.to_string(),
                serial: MonitorId::UNKNOWN_STR.to_string(),
                display_modes: modes,
                sys_path: dir,
            };

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

    info.sort_by(|a, b| b.cmp(a));

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

            DisplayMode { width, height }
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
