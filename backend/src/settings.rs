use std::path::PathBuf;

use crate::pipeline::config::{PipelineDefinition, PipelineDefinitionTemplate};

pub mod autostart;

pub struct Settings {
    dir: PathBuf,
    templates: Vec<PipelineDefinitionTemplate>,
    profiles: Vec<PipelineDefinition>,
}
