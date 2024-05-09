use std::cmp::Ordering;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DisplayValues {
    width: u16,
    height: u16,
    refresh: Option<f32>, // can't fetch it now, but I'd like to in the future if possible
}

impl Eq for DisplayValues {}

impl PartialOrd for DisplayValues {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let area = self.width * self.height;
        let other_area = other.width * other.height;

        if area < other_area {
            Some(Ordering::Less)
        } else if area > other_area {
            Some(Ordering::Greater)
        } else if self.refresh < other.refresh {
            Some(Ordering::Less)
        } else if self.refresh > other.refresh {
            Some(Ordering::Greater)
        } else {
            Some(Ordering::Equal)
        }
    }
}

impl Ord for DisplayValues {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other)
            .expect("DisplayValues should be orderable")
    }
}

/// Gets raw display info direct from the system,
/// without going through X. Primarily for displaying
/// optional values in GameMode.
///
/// Returns an optional vec of [DisplayValues], ordered
/// greatest to least.
pub fn get_display_info() -> Option<Vec<DisplayValues>> {
    // // Hardcode file for now.
    // // People *DEFINITELY* will never use a second external monitor.
    // // Or a different handheld. Definitely not.
    let file = "/sys/class/drm/card0-DP-1/modes";
    parse_modes(file)
}

fn parse_modes(file: &str) -> Option<Vec<DisplayValues>> {
    let modes = std::fs::read_to_string(file).ok()?;

    let mut modes = modes
        .split_terminator('\n')
        .into_iter()
        .filter(|v| !v.trim().is_empty())
        .map(|v| {
            let mut res = v.split_terminator('x').into_iter();
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

// We should technically parse the edid, but the file won't parse no matter which
// display I use (or parsing application/library).

// fn parse_edid(file: &str) -> Option<Vec<DisplayValues>> {
//     let bytes = std::fs::read(file).ok()?;

//     let edid = edid_rs::parse(&mut Cursor::new(bytes))
//         .inspect_err(|err| {
//             log::warn!("Error parsing EDID: {err}");
//         })
//         .ok()?;

//     let established = edid
//         .timings
//         .established_timings
//         .into_iter()
//         .map(|v| match v {
//             edid_rs::EstablishedTiming::H720V400F70 => DisplayValues {
//                 width: 720,
//                 height: 400,
//                 refresh: 70.,
//             },
//             edid_rs::EstablishedTiming::H720V400F88 => DisplayValues {
//                 width: 720,
//                 height: 400,
//                 refresh: 88.,
//             },
//             edid_rs::EstablishedTiming::H640V480F60 => DisplayValues {
//                 width: 640,
//                 height: 480,
//                 refresh: 60.,
//             },
//             edid_rs::EstablishedTiming::H640V480F67 => DisplayValues {
//                 width: 640,
//                 height: 480,
//                 refresh: 67.,
//             },
//             edid_rs::EstablishedTiming::H640V480F72 => DisplayValues {
//                 width: 640,
//                 height: 480,
//                 refresh: 72.,
//             },
//             edid_rs::EstablishedTiming::H640V480F75 => DisplayValues {
//                 width: 640,
//                 height: 480,
//                 refresh: 75.,
//             },
//             edid_rs::EstablishedTiming::H800V600F56 => DisplayValues {
//                 width: 800,
//                 height: 600,
//                 refresh: 56.,
//             },
//             edid_rs::EstablishedTiming::H800V600F60 => DisplayValues {
//                 width: 800,
//                 height: 600,
//                 refresh: 60.,
//             },
//             edid_rs::EstablishedTiming::H800V600F72 => DisplayValues {
//                 width: 800,
//                 height: 600,
//                 refresh: 72.,
//             },
//             edid_rs::EstablishedTiming::H800V600F75 => DisplayValues {
//                 width: 800,
//                 height: 600,
//                 refresh: 75.,
//             },
//             edid_rs::EstablishedTiming::H832V624F75 => DisplayValues {
//                 width: 832,
//                 height: 624,
//                 refresh: 75.,
//             },
//             edid_rs::EstablishedTiming::H1024V768F87 => DisplayValues {
//                 width: 1024,
//                 height: 768,
//                 refresh: 87.,
//             },
//             edid_rs::EstablishedTiming::H1024V768F60 => DisplayValues {
//                 width: 1024,
//                 height: 768,
//                 refresh: 60.,
//             },
//             edid_rs::EstablishedTiming::H1024V768F70 => DisplayValues {
//                 width: 1024,
//                 height: 768,
//                 refresh: 70.,
//             },
//             edid_rs::EstablishedTiming::H1024V768F75 => DisplayValues {
//                 width: 1024,
//                 height: 768,
//                 refresh: 75.,
//             },
//             edid_rs::EstablishedTiming::H1280V1024F75 => DisplayValues {
//                 width: 1280,
//                 height: 1024,
//                 refresh: 75.,
//             },
//             edid_rs::EstablishedTiming::H1152V870F75 => DisplayValues {
//                 width: 1152,
//                 height: 870,
//                 refresh: 75.,
//             },
//         });

//     let standard = edid.timings.standard_timings.into_iter().map(|v| {
//         let width = v.horizontal_resolution ;
//         DisplayValues {
//             width: width,
//             height: (width as f32 / v.aspect_ratio) as u16,
//             refresh: v.refresh_rate as f32,
//         }
//     });

//     let detailed = edid
//         .timings
//         .detailed_timings
//         .into_iter()
//         .map(|v| DisplayValues {
//             width: v.active.0,
//             height: v.active.1,
//             refresh: (v.pixel_clock as f32
//                 / (v.active.0 + v.front_porch.0 + v.back_porch.0) as f32)
//                 * (1000. / (v.active.1 + v.front_porch.1 + v.back_porch.1) as f32),
//         });

//     let mut timings = established
//         .chain(standard)
//         .chain(detailed)
//         .collect::<Vec<_>>();

//     timings.sort();
//     timings.reverse();

//     Some(timings)
// }
