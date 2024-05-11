use std::{borrow::Cow, path::Path};

use crate::{
    pipeline::{dependency::Dependency, executor::PipelineContext},
    sys::audio::AudioDeviceInfo,
};

use super::{source_file::SourceFile, ActionId, ActionImpl, ActionType};
use anyhow::{Context, Result};
use egui::TextBuffer;
use regex::Regex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CemuAudio {
    pub id: ActionId,
    // track previous devices by use, in case the desired one isn't available
    pub tv_out_device_pref: Vec<CemuAudioSetting>,
    pub pad_out_device_pref: Vec<CemuAudioSetting>,
    pub mic_in_device_pref: Vec<CemuAudioSetting>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CemuAudioState {
    pub tv_out: CemuAudioSetting,
    pub pad_out: CemuAudioSetting,
    pub mic_in: CemuAudioSetting,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CemuAudioSetting {
    pub device: AudioDeviceInfo,
    pub volume: u8,
    pub channels: CemuAudioChannels,
}

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Debug, JsonSchema)]

pub enum CemuAudioChannels {
    Mono,
    Stereo,
    Surround,
}

impl CemuAudioChannels {
    fn from_raw(raw: u8) -> Self {
        match raw {
            0 => Self::Mono,
            1 => Self::Stereo,
            _ => Self::Surround,
        }
    }

    fn to_raw(self) -> u8 {
        match self {
            Self::Mono => 0,
            Self::Stereo => 1,
            Self::Surround => 2,
        }
    }
}

lazy_static::lazy_static! {
    static ref TV_CHANNELS_RXP: Regex = Regex::new(r"<TVChannels>(\d)</TVChannels>").unwrap();
    static ref PAD_CHANNELS_RXP: Regex = Regex::new(r"<PadChannels>(\d)</PadChannels>").unwrap();
    static ref INPUT_CHANNELS_RXP: Regex = Regex::new(r"<InputChannels>(\d)</InputChannels>").unwrap();
    static ref TV_DEVICE_RXP: Regex = Regex::new(r"<TVDevice>(.*)</TVDevice>").unwrap();
    static ref PAD_DEVICE_RXP: Regex = Regex::new(r"<PadDevice>(.*)</PadDevice>").unwrap();
    static ref INPUT_DEVICE_RXP: Regex = Regex::new(r"<InputDevice>(.*)</InputDevice>").unwrap();
    static ref TV_VOLUME_RXP: Regex = Regex::new(r"<TVVolume>(\d+)</TVVolume>").unwrap();
    static ref PAD_VOLUME_RXP: Regex = Regex::new(r"<PadVolume>(\d+)</PadVolume>").unwrap();
    static ref INPUT_VOLUME_RXP: Regex = Regex::new(r"<InputVolume>(\d+)</InputVolume>").unwrap();
}

impl CemuAudioState {
    fn read<P: AsRef<Path>>(xml_path: P) -> Result<Self> {
        let xml = std::fs::read_to_string(&xml_path)?;

        // Channels
        let tv_channels = TV_CHANNELS_RXP
            .captures(&xml)
            .expect("settings.xml should have open_pad setting")
            .get(1)
            .with_context(|| "TV_CHANNELS_RXP should have one capture")?;
        let pad_channels = PAD_CHANNELS_RXP
            .captures(&xml)
            .expect("settings.xml should have open_pad setting")
            .get(1)
            .with_context(|| "PAD_CHANNELS_RXP should have one capture")?;
        let input_channels = INPUT_CHANNELS_RXP
            .captures(&xml)
            .expect("settings.xml should have open_pad setting")
            .get(1)
            .with_context(|| "INPUT_CHANNELS_RXP rxp should have one capture")?;

        // Volume
        let tv_volume = TV_VOLUME_RXP
            .captures(&xml)
            .expect("settings.xml should have open_pad setting")
            .get(1)
            .with_context(|| "TV_VOLUME_RXP should have one capture")?;
        let pad_volume = PAD_VOLUME_RXP
            .captures(&xml)
            .expect("settings.xml should have open_pad setting")
            .get(1)
            .with_context(|| "PAD_VOLUME_RXP should have one capture")?;
        let input_volume = INPUT_VOLUME_RXP
            .captures(&xml)
            .expect("settings.xml should have open_pad setting")
            .get(1)
            .with_context(|| "INPUT_VOLUME_RXP rxp should have one capture")?;

        // Device
        let tv_device = TV_DEVICE_RXP
            .captures(&xml)
            .expect("settings.xml should have open_pad setting")
            .get(1)
            .with_context(|| "TV_DEVICE_RXP should have one capture")?;
        let pad_device = PAD_DEVICE_RXP
            .captures(&xml)
            .expect("settings.xml should have open_pad setting")
            .get(1)
            .with_context(|| "PAD_DEVICE_RXP should have one capture")?;
        let input_device = INPUT_DEVICE_RXP
            .captures(&xml)
            .expect("settings.xml should have open_pad setting")
            .get(1)
            .with_context(|| "INPUT_DEVICE_RXP rxp should have one capture")?;

        Ok(Self {
            tv_out: CemuAudioSetting {
                device: AudioDeviceInfo::from_name(tv_device.as_str().to_string()),
                volume: tv_volume.as_str().parse().unwrap(),
                channels: CemuAudioChannels::from_raw(tv_channels.as_str().parse().unwrap()),
            },
            pad_out: CemuAudioSetting {
                device: AudioDeviceInfo::from_name(pad_device.as_str().to_string()),
                volume: pad_volume.as_str().parse().unwrap(),
                channels: CemuAudioChannels::from_raw(pad_channels.as_str().parse().unwrap()),
            },
            mic_in: CemuAudioSetting {
                device: AudioDeviceInfo::from_name(input_device.as_str().to_string()),
                volume: input_volume.as_str().parse().unwrap(),
                channels: CemuAudioChannels::from_raw(input_channels.as_str().parse().unwrap()),
            },
        })
    }

    fn write<P: AsRef<Path>>(&self, xml_path: P) -> Result<()> {
        let mut xml = std::fs::read_to_string(&xml_path)?;

        let options: [(&Regex, &Regex, &Regex, &CemuAudioSetting); 3] = [
            (
                &TV_DEVICE_RXP,
                &TV_VOLUME_RXP,
                &TV_CHANNELS_RXP,
                &self.tv_out,
            ),
            (
                &PAD_DEVICE_RXP,
                &PAD_VOLUME_RXP,
                &PAD_CHANNELS_RXP,
                &self.pad_out,
            ),
            (
                &INPUT_DEVICE_RXP,
                &INPUT_VOLUME_RXP,
                &INPUT_CHANNELS_RXP,
                &self.mic_in,
            ),
        ];

        for (device_rxp, volume_rxp, channels_rxp, value) in options {
            let xml1 = device_rxp.replace(&xml, &value.device.name);
            let xml2 = volume_rxp.replace(&xml1, &value.volume.to_string());
            let xml3 = channels_rxp.replace(&xml2, &value.channels.to_raw().to_string());

            xml = xml3.to_string();
        }

        Ok(std::fs::write(xml_path, xml.as_str())?)
    }
}

impl ActionImpl for CemuAudio {
    type State = CemuAudioState;

    const TYPE: ActionType = ActionType::CemuAudio;

    fn setup(&self, ctx: &mut PipelineContext) -> Result<()> {
        let (xml_path, audio) = {
            let xml_path = ctx
                .get_state::<SourceFile>()
                .with_context(|| "No source file set for Cemu settings")?;

            (xml_path, CemuAudioState::read(xml_path)?)
        };

        todo!()
        // self.state.write(xml_path).map(|_| {
        //     ctx.set_state::<Self>(Audio);
        // })
    }

    fn teardown(&self, ctx: &mut PipelineContext) -> Result<()> {
        let state = ctx.get_state::<Self>();

        match state {
            Some(state) => {
                let xml_path = ctx
                    .get_state::<SourceFile>()
                    .with_context(|| "No source file set for Cemu settings")?;

                state.write(xml_path)
            }
            None => Ok(()),
        }
    }

    fn get_dependencies(
        &self,
        _ctx: &PipelineContext,
    ) -> Vec<crate::pipeline::dependency::Dependency> {
        vec![Dependency::System("pactl".to_string())]
    }

    #[inline]
    fn get_id(&self) -> ActionId {
        self.id
    }
}

// #[cfg(test)]
// mod tests {
//     use std::path::PathBuf;

//     use crate::util::create_dir_all;

//     use pretty_assertions::assert_eq;

//     use super::*;

//     #[test]
//     fn test_read_write_cemu_Audio() -> Result<()> {
//         let source_path = "test/assets/cemu/settings.xml";
//         let source = std::fs::read_to_string(source_path)?;
//         let path = PathBuf::from("test/out/cemu/settings.xml");
//         create_dir_all(path.parent().unwrap())?;

//         std::fs::write(&path, &source)?;

//         let expected = CemuAudioState {
//             separate_gamepad_view: false,
//             fullscreen: false,
//         };
//         let actual = CemuAudioState::read(&path)?;

//         assert_eq!(expected, actual);

//         expected.write(&path)?;
//         let actual_str = std::fs::read_to_string(&path)?;
//         assert_eq!(source, actual_str);

//         let expected = CemuAudioState {
//             separate_gamepad_view: true,
//             fullscreen: true,
//         };

//         expected.write(&path)?;

//         let actual = CemuAudioState::read(&path)?;

//         assert_eq!(expected, actual);

//         std::fs::remove_file(path)?;
//         Ok(())
//     }
// }
