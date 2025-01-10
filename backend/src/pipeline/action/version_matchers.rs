use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::pipeline::{data::VersionMatcher, executor::PipelineContext};

use super::emu_source::EmuSettingsSourceConfig;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
enum MelonDSVersion {
    Prerelease,
    V1,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct MelonDSVersionMatcher(MelonDSVersion);

impl MelonDSVersionMatcher {
    pub fn prerelease() -> Self {
        Self(MelonDSVersion::Prerelease)
    }

    pub fn v1() -> Self {
        Self(MelonDSVersion::V1)
    }
}

#[typetag::serde]
impl VersionMatcher for MelonDSVersionMatcher {
    fn matches_version(&self, ctx: &PipelineContext) -> Result<bool> {
        let state = ctx
            .get_state::<EmuSettingsSourceConfig>()
            .context("melonDS version matching requires settings source in pipeline")?;

        let res = match self.0 {
            MelonDSVersion::Prerelease => state.ends_with(".ini"),
            MelonDSVersion::V1 => state.ends_with(".toml"),
        };

        Ok(res)
    }
}
