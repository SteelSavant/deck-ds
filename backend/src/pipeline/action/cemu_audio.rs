use std::path::Path;

use crate::{
    pipeline::{dependency::Dependency, executor::PipelineContext},
    sys::audio::{get_audio_sinks, get_audio_sources},
};

use super::{source_file::SourceFile, ActionId, ActionImpl, ActionType};
use anyhow::{Context, Result};
use regex::Regex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CemuAudio {
    pub id: ActionId,
    pub state: CemuAudioState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CemuAudioState {
    pub tv_out: CemuAudioSetting,
    pub pad_out: CemuAudioSetting,
    pub mic_in: CemuAudioSetting,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CemuAudioSetting {
    pub device: String,
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
            .expect("settings.xml should have TVChannels setting")
            .get(1)
            .with_context(|| "TV_CHANNELS_RXP should have one capture")?;
        let pad_channels = PAD_CHANNELS_RXP
            .captures(&xml)
            .expect("settings.xml should have PadChannels setting")
            .get(1)
            .with_context(|| "PAD_CHANNELS_RXP should have one capture")?;
        let input_channels = INPUT_CHANNELS_RXP
            .captures(&xml)
            .expect("settings.xml should have InputChannels setting")
            .get(1)
            .with_context(|| "INPUT_CHANNELS_RXP rxp should have one capture")?;

        // Volume
        let tv_volume = TV_VOLUME_RXP
            .captures(&xml)
            .expect("settings.xml should have TVVolume setting")
            .get(1)
            .with_context(|| "TV_VOLUME_RXP should have one capture")?;
        let pad_volume = PAD_VOLUME_RXP
            .captures(&xml)
            .expect("settings.xml should have PadVolume setting")
            .get(1)
            .with_context(|| "PAD_VOLUME_RXP should have one capture")?;
        let input_volume = INPUT_VOLUME_RXP
            .captures(&xml)
            .expect("settings.xml should have InputVolume setting")
            .get(1)
            .with_context(|| "INPUT_VOLUME_RXP rxp should have one capture")?;

        // Device
        let tv_device = TV_DEVICE_RXP
            .captures(&xml)
            .expect("settings.xml should have TVDevice setting")
            .get(1)
            .with_context(|| "TV_DEVICE_RXP should have one capture")?;
        let pad_device = PAD_DEVICE_RXP
            .captures(&xml)
            .expect("settings.xml should have PadDevice setting")
            .get(1)
            .with_context(|| "PAD_DEVICE_RXP should have one capture")?;
        let input_device = INPUT_DEVICE_RXP
            .captures(&xml)
            .expect("settings.xml should have InputDevice setting")
            .get(1)
            .with_context(|| "INPUT_DEVICE_RXP rxp should have one capture")?;

        Ok(Self {
            tv_out: CemuAudioSetting {
                device: tv_device.as_str().to_string(),
                volume: tv_volume.as_str().parse().unwrap(),
                channels: CemuAudioChannels::from_raw(tv_channels.as_str().parse().unwrap()),
            },
            pad_out: CemuAudioSetting {
                device: pad_device.as_str().to_string(),
                volume: pad_volume.as_str().parse().unwrap(),
                channels: CemuAudioChannels::from_raw(pad_channels.as_str().parse().unwrap()),
            },
            mic_in: CemuAudioSetting {
                device: input_device.as_str().to_string(),
                volume: input_volume.as_str().parse().unwrap(),
                channels: CemuAudioChannels::from_raw(input_channels.as_str().parse().unwrap()),
            },
        })
    }

    fn write<P: AsRef<Path>>(&self, xml_path: P) -> Result<()> {
        let mut xml = std::fs::read_to_string(&xml_path)?;

        let options: [(&str, &Regex, &Regex, &Regex, &CemuAudioSetting); 3] = [
            (
                "TV",
                &TV_DEVICE_RXP,
                &TV_VOLUME_RXP,
                &TV_CHANNELS_RXP,
                &self.tv_out,
            ),
            (
                "Pad",
                &PAD_DEVICE_RXP,
                &PAD_VOLUME_RXP,
                &PAD_CHANNELS_RXP,
                &self.pad_out,
            ),
            (
                "Input",
                &INPUT_DEVICE_RXP,
                &INPUT_VOLUME_RXP,
                &INPUT_CHANNELS_RXP,
                &self.mic_in,
            ),
        ];

        for (tag, device_rxp, volume_rxp, channels_rxp, value) in options {
            let out = format!("<{tag}Device>{}</{tag}Device>", value.device);
            let xml1 = device_rxp.replace(&xml, &out);
            let out = format!("<{tag}Volume>{}</{tag}Volume>", value.volume);
            let xml2 = volume_rxp.replace(&xml1, &out);
            let out = format!("<{tag}Channels>{}</{tag}Channels>", value.channels.to_raw());
            let xml3 = channels_rxp.replace(&xml2, &out);

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

        let sources = get_audio_sources(&ctx.decky_env);
        let sinks = get_audio_sinks(&ctx.decky_env);

        let mut state = self.state.clone();
        let available_tv_out = sinks
            .iter()
            .map(|s| s.name.as_str())
            .chain(["default", ""])
            .any(|s| *s == state.tv_out.device);

        let available_pad_out = sinks
            .iter()
            .map(|s| s.name.as_str())
            .chain(["default", ""])
            .any(|s| *s == state.pad_out.device);

        let available_mic_in = sources
            .iter()
            .map(|s| s.name.as_str())
            .chain(["default", ""])
            .any(|s| *s == state.mic_in.device);

        // TODO::if audio.*.device is empty, replace with default

        if !available_tv_out {
            state.tv_out.device = audio.tv_out.device.clone();
        }

        if !available_pad_out {
            state.pad_out.device = audio.pad_out.device.clone();
        }

        if !available_mic_in {
            state.mic_in.device = audio.mic_in.device.clone();
        }

        state.write(xml_path).map(|_| {
            ctx.set_state::<Self>(audio);
        })
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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::util::create_dir_all;

    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_read_write_cemu_audio() -> Result<()> {
        let source_path = "test/assets/cemu/settings.xml";
        let source = std::fs::read_to_string(source_path)?;
        let path = PathBuf::from("test/out/cemu/audio_settings.xml");
        create_dir_all(path.parent().unwrap())?;

        std::fs::write(&path, &source)?;

        let expected = CemuAudioState {
            tv_out: CemuAudioSetting {
                device: "default".to_string(),
                volume: 50,
                channels: CemuAudioChannels::Surround,
            },
            pad_out: CemuAudioSetting {
                device: "".to_string(),
                volume: 0,
                channels: CemuAudioChannels::Stereo,
            },
            mic_in: CemuAudioSetting {
                device: "".to_string(),
                volume: 20,
                channels: CemuAudioChannels::Mono,
            },
        };
        let actual = CemuAudioState::read(&path)?;

        assert_eq!(expected, actual);

        expected.write(&path)?;
        let actual_str = std::fs::read_to_string(&path)?;
        assert_eq!(source, actual_str);

        let actual = CemuAudioState::read(&path)?;
        assert_eq!(expected, actual);

        std::fs::remove_file(path)?;
        Ok(())
    }
}
