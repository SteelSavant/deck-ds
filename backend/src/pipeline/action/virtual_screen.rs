use indexmap::IndexMap;

use crate::pipeline::{common::{Context, PipelineActionExecutor, PipelineActionExecutorId}, config::PipelineAction};

use super::super::common::Dependency;
pub struct VirtualScreen;


impl PipelineActionExecutor for VirtualScreen {
    fn id(&self) -> PipelineActionExecutorId { PipelineActionExecutorId("VirtualScreen".to_string()) }
    fn definition(&self) -> PipelineAction {
        PipelineAction {
            name: self.id().0,
            args: IndexMap::new()
        }
    }

    fn get_dependencies(&self) -> Vec<Dependency> {
        vec![Dependency::TrueVideoWall]
    }
}
