use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};

use crate::pipeline::executor::PipelineContext;

use super::PipelineActionImpl;
use anyhow::Result;
use regex::Regex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum CemuXmlSource {
    Flatpak,
    Custom(PathBuf),
    // AppImage,
    // EmuDeckExe,
}

impl CemuXmlSource {
    pub fn get_path<P: AsRef<Path>>(&self, home_dir: P) -> Result<Cow<PathBuf>> {
        Ok(match self {
            CemuXmlSource::Flatpak => Cow::Owned(
                home_dir
                    .as_ref()
                    .join(".var/app/info.cemu.Cemu/config/Cemu/settings.xml"),
            ),
            CemuXmlSource::Custom(path) => Cow::Borrowed(path),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CemuConfig {
    pub xml_source: CemuXmlSource,
    pub separate_gamepad_view: bool,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CemuState {
    pub separate_gamepad_view: bool,
}

lazy_static::lazy_static! {
    static ref RXP: Regex =  Regex::new("<open_pad>((?:true)|(?:false))</open_pad>").unwrap();
}

impl PipelineActionImpl for CemuConfig {
    type State = CemuState;

    fn setup(&self, ctx: &mut PipelineContext) -> Result<()> {
        let xml_path = self.get_xml_path(&ctx.home_dir)?;
        let xml = std::fs::read_to_string(xml_path.as_path())?;

        let current = RXP
            .captures(&xml)
            .expect("settings.xml should have open_pad setting")
            .get(1)
            .expect("rxp should have one capture");

        ctx.set_state::<Self>(CemuState {
            separate_gamepad_view: current.as_str() == "true",
        });

        let out = format!("<open_pad>{}</open_pad>", self.separate_gamepad_view);

        std::fs::write(xml_path.as_path(), RXP.replace(&xml, out).as_ref())?;

        Ok(())
    }

    fn teardown(&self, ctx: &mut PipelineContext) -> Result<()> {
        let state = ctx.get_state::<Self>();

        match state {
            Some(state) => {
                let xml_path = self.get_xml_path(&ctx.home_dir)?;
                let xml = std::fs::read_to_string(xml_path.as_path())?;

                let out = format!("<open_pad>{}</open_pad>", state.separate_gamepad_view);

                Ok(std::fs::write(
                    xml_path.as_path(),
                    RXP.replace(&xml, out).as_ref(),
                )?)
            }
            None => Ok(()),
        }
    }
}

impl CemuConfig {
    fn get_xml_path<P: AsRef<Path>>(&self, home_dir: P) -> Result<Cow<PathBuf>> {
        let xml_path = self.xml_source.get_path(home_dir)?;
        if xml_path.is_file() {
            Ok(xml_path)
        } else {
            Err(std::io::Error::from(std::io::ErrorKind::NotFound).into())
        }
    }
}
