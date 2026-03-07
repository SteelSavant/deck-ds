use std::{
    cmp::Ordering,
    collections::HashMap,
    fs::DirEntry,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Context};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

use crate::macros::newtype_strid;

newtype_strid!("Display Id", DisplayId);

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
pub struct DisplayInfo {
    vendor: String,
    product: u16,
    serial: u32,
    serial_number: Option<String>,
    product_name: Option<String>,
    pub values: Vec<DisplayValues>,
}

pub struct RuntimeDisplayInfo {
    pub enabled: bool,
    pub path: String,
}

impl DisplayInfo {
    pub fn id(&self) -> DisplayId {
        DisplayId(format!("{}-{}-{}", self.vendor, self.product, self.serial))
    }

    pub fn display_name(&self) -> String {
        let product = self
            .product_name
            .clone()
            .unwrap_or(format!("{}-{}", self.vendor, self.product));
        let serial = self
            .serial_number
            .clone()
            .unwrap_or(self.serial.to_string());

        format!("{product}_{serial}")
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DisplayValues {
    width: u16,
    height: u16,
    refresh: Option<f32>, // can't fetch it now, but I'd like to in the future if possible/practical/sane
}

impl Eq for DisplayValues {}

impl PartialOrd for DisplayValues {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DisplayValues {
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
/// optional values in GameMode.
///
/// Returns a vec of [DisplayInfo] and a hashmap mapping internal [DisplayId]s to device path (for later remapping)
pub fn get_display_info() -> (Vec<DisplayInfo>, HashMap<DisplayId, RuntimeDisplayInfo>) {
    let mut runtime_info: HashMap<DisplayId, _> = HashMap::new();
    let info = get_display_dirs()
        .into_iter()
        .map(|dir| {
            let modes_file = dir.join("modes");
            let edid_file = dir.join("edid");

            let modes = parse_modes(modes_file);
            let mut edid = parse_edid(edid_file)
                .inspect_err(|err| log::warn!("Failed to parse edid: {err}"))
                .unwrap_or_default();
            if let Some(mut modes) = modes {
                edid.values.append(&mut modes);
                edid.values.dedup();
            }

            let enabled = std::fs::read_to_string(dir.join("enabled"))
                .unwrap_or("disabled".to_string())
                == "enabled";

            runtime_info.insert(
                edid.id(),
                RuntimeDisplayInfo {
                    enabled: enabled,
                    path: dir
                        .parent()
                        .into_iter()
                        .last()
                        .unwrap()
                        .to_string_lossy()
                        .to_string(),
                },
            );

            edid
        })
        .collect();

    (info, runtime_info)
}

fn get_display_dirs() -> Vec<PathBuf> {
    let drm_dir = Path::new("/sys/class/drm/");
    fn is_connected_display_dir(d: &DirEntry) -> bool {
        let dir_path = d.path();
        let exists = dir_path.join("modes").exists() || dir_path.join("edid").exists();
        let status =
            std::fs::read_to_string(dir_path.join("status")).unwrap_or("disconnected".to_string());

        exists && status == "connected"
    }

    match drm_dir.read_dir() {
        Ok(display_dirs) => display_dirs
            .filter_map(Result::ok)
            .filter(|v| {
                !v.file_name()
                    .to_string_lossy()
                    .to_lowercase()
                    .contains("writeback")
            })
            .filter(is_connected_display_dir)
            .map(|d| d.path())
            .collect(),
        Err(_) => vec![],
    }
}

fn parse_modes<P: AsRef<Path>>(file: P) -> Option<Vec<DisplayValues>> {
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

            DisplayValues {
                width,
                height,
                refresh: None,
            }
        })
        .collect::<Vec<_>>();

    modes.dedup();

    Some(modes)
}

pub fn parse_raw_edid(bytes: &[u8]) -> Result<DisplayInfo, anyhow::Error> {
    let edid = edid::parse(&bytes);
    let edid = edid
        .to_result()
        .map_err(|_| anyhow!("Failed to parse edid"));

    let edid = edid.with_context(|| format!("Failed to parse edid"))?;

    let mut info = DisplayInfo {
        vendor: edid.header.vendor.iter().collect(),
        product: edid.header.product,
        serial: edid.header.serial,
        product_name: None,
        serial_number: None,
        values: vec![],
    };

    for d in edid.descriptors {
        match d {
            edid::Descriptor::SerialNumber(s) => info.serial_number = Some(s),
            edid::Descriptor::ProductName(p) => info.product_name = Some(p),
            edid::Descriptor::DetailedTiming(d) => {
                // let h_total = d.horizontal_active_pixels + d.horizontal_blanking_pixels;
                // let v_total = d.vertical_active_lines + d.vertical_blanking_lines;
                // let refresh = (d.pixel_clock as f32) / (h_total + v_total) as f32;

                info.values.push(DisplayValues {
                    width: d.horizontal_active_pixels,
                    height: d.vertical_active_lines,
                    refresh: None, // skip refresh for now, values don't well match actual monitor support
                });
            }
            _ => {}
        }
    }

    Ok(info)
}

fn parse_edid<P: AsRef<Path> + Debug>(path: P) -> Result<DisplayInfo, anyhow::Error> {
    let bytes = std::fs::read(&path)?;

    parse_raw_edid(&bytes).with_context(|| format!("Failed to parse edid file @ {:?}", path))
}
