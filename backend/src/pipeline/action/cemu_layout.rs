use std::path::Path;

use crate::pipeline::executor::PipelineContext;

use super::{source_file::SourceFile, ActionImpl};
use anyhow::{Context, Result};
use regex::Regex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CemuLayout {
    pub separate_gamepad_view: bool,
    pub swap_screens: bool,
}

lazy_static::lazy_static! {
    static ref RXP: Regex =  Regex::new("<open_pad>((?:true)|(?:false))</open_pad>").unwrap();
}

impl CemuLayout {
    fn read<P: AsRef<Path>>(xml_path: P) -> Result<Self> {
        let xml = std::fs::read_to_string(&xml_path)?;

        let current = RXP
            .captures(&xml)
            .expect("settings.xml should have open_pad setting")
            .get(1)
            .with_context(|| "open_pad rxp should have one capture")?;

        Ok(Self {
            separate_gamepad_view: current.as_str() == "true",
            swap_screens: false, // TODO
        })
    }

    fn write<P: AsRef<Path>>(&self, xml_path: P) -> Result<()> {
        let xml = std::fs::read_to_string(&xml_path)?;

        let out = format!("<open_pad>{}</open_pad>", self.separate_gamepad_view);

        Ok(std::fs::write(xml_path, RXP.replace(&xml, out).as_ref())?)
    }
}

impl ActionImpl for CemuLayout {
    type State = CemuLayout;

    fn setup(&self, ctx: &mut PipelineContext) -> Result<()> {
        let (xml_path, layout) = {
            let xml_path = ctx
                .get_state::<SourceFile>()
                .with_context(|| "No source file set for Cemu settings")?;

            (xml_path, CemuLayout::read(&xml_path)?)
        };

        self.write(xml_path).map(|_| {
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
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_read_cemu_layout() {
        todo!()
    }

    #[test]
    fn test_write_cemu_layout() {
        todo!()
    }
}
