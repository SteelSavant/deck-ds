use std::path::Path;

use crate::pipeline::executor::PipelineContext;

use super::{source_file::SourceFile, ActionId, ActionImpl, ActionType};
use anyhow::{Context, Result};
use regex::Regex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CemuLayout {
    pub id: ActionId,
    pub layout: CemuLayoutState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CemuLayoutState {
    pub separate_gamepad_view: bool,
    pub fullscreen: bool,
}

lazy_static::lazy_static! {
    static ref PAD_RXP: Regex = Regex::new("<open_pad>((?:true)|(?:false))</open_pad>").unwrap();
    static ref FULLSCREEN_RXP: Regex = Regex::new("<fullscreen>((?:true)|(?:false))</fullscreen>").unwrap();
}

impl CemuLayoutState {
    fn read<P: AsRef<Path>>(xml_path: P) -> Result<Self> {
        let xml = std::fs::read_to_string(&xml_path)?;

        let current_open_pad = PAD_RXP
            .captures(&xml)
            .expect("settings.xml should have open_pad setting")
            .get(1)
            .with_context(|| "open_pad rxp should have one capture")?;

        let current_fullscreen = FULLSCREEN_RXP
            .captures(&xml)
            .expect("settings.xml should have fullscreen setting")
            .get(1)
            .with_context(|| "fullscreen rxp should have one capture")?;

        Ok(Self {
            separate_gamepad_view: current_open_pad.as_str() == "true",
            fullscreen: current_fullscreen.as_str() == "true",
        })
    }

    fn write<P: AsRef<Path>>(&self, xml_path: P) -> Result<()> {
        let xml = std::fs::read_to_string(&xml_path)?;

        let out = format!("<open_pad>{}</open_pad>", self.separate_gamepad_view);
        let replaced_pad = PAD_RXP.replace(&xml, &out);

        let out = format!("<fullscreen>{}</fullscreen>", self.fullscreen);
        let replaced_fullscreen = FULLSCREEN_RXP.replace(&replaced_pad, &out);

        Ok(std::fs::write(xml_path, replaced_fullscreen.as_ref())?)
    }
}

impl ActionImpl for CemuLayout {
    type State = CemuLayoutState;

    const TYPE: ActionType = ActionType::CemuLayout;

    fn setup(&self, ctx: &mut PipelineContext) -> Result<()> {
        let (xml_path, layout) = {
            let xml_path = ctx
                .get_state::<SourceFile>()
                .with_context(|| "No source file set for Cemu settings")?;

            (xml_path, CemuLayoutState::read(xml_path)?)
        };

        self.layout.write(xml_path).map(|_| {
            ctx.set_state::<Self>(layout);
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
    fn test_read_write_cemu_layout() -> Result<()> {
        let source_path = "test/assets/cemu/settings.xml";
        let source = std::fs::read_to_string(source_path)?;
        let path = PathBuf::from("test/out/cemu/settings.xml");
        create_dir_all(path.parent().unwrap())?;

        std::fs::write(&path, &source)?;

        let expected = CemuLayoutState {
            separate_gamepad_view: false,
            fullscreen: false,
        };
        let actual = CemuLayoutState::read(&path)?;

        assert_eq!(expected, actual);

        expected.write(&path)?;
        let actual_str = std::fs::read_to_string(&path)?;
        assert_eq!(source, actual_str);

        let expected = CemuLayoutState {
            separate_gamepad_view: true,
            fullscreen: true,
        };

        expected.write(&path)?;

        let actual = CemuLayoutState::read(&path)?;

        assert_eq!(expected, actual);

        std::fs::remove_file(path)?;
        Ok(())
    }
}
